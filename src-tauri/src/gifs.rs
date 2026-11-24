//! GIF search on Tenor, for memes that aren't in the library yet.
//!
//! Only runs when you ask for it (a `!gif` search, or "Search Tenor" from the pane), so
//! nothing leaves the device otherwise. Results can be pasted or copied straight away
//! (without saving them) or added to the library like any other import.

use crate::import::{self, ImportResult};
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;
use tauri::AppHandle;

/// Tenor's v1 API. Google closed the v2 API to new keys, so this uses the key from the
/// Gboard iOS app, as Vencord does (src/plugins/gifProviderSwitcher/tenor.ts).
const TENOR_API: &str = "https://api.tenor.com/v1";
const TENOR_KEY: &str = "3Z0688EVWYKH";
/// Tenor's page size limit.
const PAGE: u32 = 50;
const MAX_QUERY: usize = 100;
/// Tenor's API answers are small JSON; refuse anything absurd.
const MAX_RESPONSE: usize = 4 * 1024 * 1024;

#[derive(Serialize, Debug, PartialEq)]
pub struct Gif {
    pub id: String,
    pub title: String,
    /// The full-size GIF (what gets pasted, copied or saved).
    pub url: String,
    /// A small animated version, for the grid.
    pub preview: String,
    /// A still frame of the small version, for tiles that aren't playing.
    pub still: String,
    /// Size of the small version.
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Default)]
pub struct GifPage {
    pub gifs: Vec<Gif>,
    /// Pass back as `pos` for the next page; None when there are no more.
    pub next: Option<String>,
}

/// Only Tenor's own media servers, over https.
fn tenor_media(url: &str) -> Option<reqwest::Url> {
    let parsed = reqwest::Url::parse(url).ok()?;
    let host = parsed.host_str()?;
    let ok = parsed.scheme() == "https"
        && parsed.username().is_empty()
        && parsed.password().is_none()
        && parsed.port().is_none()
        && (host == "media.tenor.com" || host == "c.tenor.com" || is_numbered_media(host));
    ok.then_some(parsed)
}

/// media1.tenor.com, media2.tenor.com, ...
fn is_numbered_media(host: &str) -> bool {
    host.strip_suffix(".tenor.com")
        .and_then(|h| h.strip_prefix("media"))
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

fn dims(v: &Value) -> Option<(u32, u32)> {
    let d = v.get("dims")?.as_array()?;
    let w = u32::try_from(d.first()?.as_u64()?).ok()?;
    let h = u32::try_from(d.get(1)?.as_u64()?).ok()?;
    (w > 0 && h > 0 && w <= 10_000 && h <= 10_000).then_some((w, h))
}

/// One Tenor result, or None if anything we need is missing or off-site.
fn parse_result(item: &Value) -> Option<Gif> {
    let id = item.get("id")?.as_str()?;
    if id.is_empty() || id.len() > 100 || !id.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return None;
    }
    let media = item.get("media")?.as_array()?.first()?;
    let gif = media.get("gif")?;
    let tiny = media.get("tinygif").unwrap_or(gif);
    let url = tenor_media(gif.get("url")?.as_str()?)?;
    let preview = tenor_media(tiny.get("url")?.as_str()?)?;
    let still = tiny
        .get("preview")
        .and_then(Value::as_str)
        .and_then(tenor_media)
        .unwrap_or_else(|| preview.clone());
    let (width, height) = dims(tiny).or_else(|| dims(gif))?;

    let text = |key: &str| item.get(key).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty());
    let title = text("title")
        .or_else(|| text("h1_title").map(|t| t.strip_suffix(" GIF").unwrap_or(t)))
        .or_else(|| text("content_description"))
        .unwrap_or("GIF");
    let title: String = title.chars().filter(|c| !c.is_control()).take(120).collect();

    Some(Gif {
        id: id.to_string(),
        title,
        url: url.into(),
        preview: preview.into(),
        still: still.into(),
        width,
        height,
    })
}

fn parse_page(body: &Value) -> Result<GifPage, String> {
    let results = body
        .get("results")
        .and_then(Value::as_array)
        .ok_or("Tenor sent back something unexpected.")?;
    let gifs: Vec<Gif> = results.iter().filter_map(parse_result).collect();
    // Tenor sends `next` even on the last page; an empty page means we're done.
    let next = body
        .get("next")
        .and_then(Value::as_str)
        .filter(|n| !n.is_empty() && n.len() < 200 && *n != "0" && !results.is_empty())
        .map(str::to_owned);
    Ok(GifPage { gifs, next })
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("memeity (https://memeity.stelle.codes)")
        .timeout(Duration::from_secs(10))
        .build()
        .expect("http client")
}

/// Searches Tenor (or lists trending GIFs, for an empty query).
#[tauri::command]
pub async fn gif_search(query: String, pos: Option<String>) -> Result<GifPage, String> {
    let query = query.split_whitespace().collect::<Vec<_>>().join(" ");
    if query.chars().count() > MAX_QUERY {
        return Err(format!("Keep GIF searches under {MAX_QUERY} characters."));
    }
    let endpoint = if query.is_empty() { "trending" } else { "search" };
    let mut url = reqwest::Url::parse(&format!("{TENOR_API}/{endpoint}")).expect("tenor url");
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("key", TENOR_KEY)
            .append_pair("limit", &PAGE.to_string())
            .append_pair("media_filter", "minimal")
            .append_pair("contentfilter", "low");
        if !query.is_empty() {
            q.append_pair("q", &query);
        }
        if let Some(pos) = pos.as_deref().filter(|p| !p.is_empty()) {
            q.append_pair("pos", pos);
        }
    }

    let resp = client()
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Couldn't reach Tenor: {}", e.without_url()))?;
    if !resp.status().is_success() {
        return Err(format!("Tenor search failed ({}).", resp.status()));
    }
    let bytes = resp.bytes().await.map_err(|e| format!("Couldn't reach Tenor: {}", e.without_url()))?;
    if bytes.len() > MAX_RESPONSE {
        return Err("Tenor sent back too much.".into());
    }
    let body: Value = serde_json::from_slice(&bytes).map_err(|_| "Tenor sent back something unexpected.".to_string())?;
    parse_page(&body)
}

async fn download(url: &str) -> Result<Vec<u8>, String> {
    let url = tenor_media(url).ok_or("That isn't a Tenor GIF.")?;
    let (bytes, _, _) = import::fetch(&import::client(), &url).await?;
    Ok(bytes)
}

fn file_name(title: &str) -> String {
    let title = title.trim();
    format!("{}.gif", if title.is_empty() { "GIF" } else { title })
}

/// Pastes a Tenor GIF into the app you were in (`paste`), or copies it, without adding
/// it to the library.
#[tauri::command]
pub async fn gif_share(app: AppHandle, url: String, title: String, paste: bool) -> Result<(), String> {
    let bytes = download(&url).await?;
    let path = crate::clipboard::temp_file(&file_name(&title), &bytes)?;
    crate::share_file(&app, path, paste).await
}

/// Adds a Tenor GIF to the library.
#[tauri::command]
pub async fn gif_save(app: AppHandle, url: String, title: String) -> Result<ImportResult, String> {
    let bytes = download(&url).await?;
    let result = import::add_bytes(&app, Some(&file_name(&title)), &bytes)?;
    import::finish(&app, &result);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn only_tenor_media_hosts() {
        assert!(tenor_media("https://media.tenor.com/a/b.gif").is_some());
        assert!(tenor_media("https://media1.tenor.com/a/b.gif").is_some());
        assert!(tenor_media("https://c.tenor.com/a/b.gif").is_some());
        assert!(tenor_media("http://media.tenor.com/a/b.gif").is_none());
        assert!(tenor_media("https://media.tenor.com.evil.com/b.gif").is_none());
        assert!(tenor_media("https://mediax.tenor.com/b.gif").is_none());
        assert!(tenor_media("https://user@media.tenor.com/b.gif").is_none());
        assert!(tenor_media("https://media.tenor.com:8443/b.gif").is_none());
        assert!(tenor_media("https://tenor.com/view/cat").is_none());
    }

    #[test]
    fn parses_tenor_v1_results() {
        let body = json!({
            "next": "2",
            "results": [
                {
                    "id": "7523542997650649169",
                    "title": "",
                    "h1_title": "Cat Dance GIF",
                    "media": [{
                        "tinygif": {
                            "url": "https://media.tenor.com/x/cat-dance.gif",
                            "preview": "https://media.tenor.com/y/cat-dance.png",
                            "dims": [220, 211]
                        },
                        "gif": { "url": "https://media.tenor.com/z/cat-dance.gif", "dims": [498, 476] }
                    }]
                },
                // Off-site media is dropped.
                {
                    "id": "1",
                    "title": "bad",
                    "media": [{ "gif": { "url": "https://evil.example/a.gif", "dims": [1, 1] } }]
                }
            ]
        });
        let page = parse_page(&body).unwrap();
        assert_eq!(page.next.as_deref(), Some("2"));
        assert_eq!(
            page.gifs,
            vec![Gif {
                id: "7523542997650649169".into(),
                title: "Cat Dance".into(),
                url: "https://media.tenor.com/z/cat-dance.gif".into(),
                preview: "https://media.tenor.com/x/cat-dance.gif".into(),
                still: "https://media.tenor.com/y/cat-dance.png".into(),
                width: 220,
                height: 211,
            }]
        );
        let empty = parse_page(&json!({ "next": "50", "results": [] })).unwrap();
        assert!(empty.next.is_none());
        assert!(parse_page(&json!({ "error": "nope" })).is_err());
    }
}
