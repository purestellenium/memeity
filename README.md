<h1 align="center">
  <img src="./src/assets/logo.png" alt="logo" width="200" />
  <br />memeity
</h1>
<h3 align="center">a meme folder for the modern era.</h3>

<p align="center"><i>hey, it's me, it's memeity</i></p>

<div align="center">
  <img alt="GitHub Release" src="https://img.shields.io/github/downloads/purestellenium/memeity/total">
  <img alt="GitHub Downloads (all assets, all releases)" src="https://img.shields.io/github/v/release/purestellenium/memeity">
  <img alt="GitHub License" src="https://img.shields.io/github/license/purestellenium/memeity?color=orange">
</div>

<br />

<div align="center">
  <img width="759" height="486" alt="screenshot" src="https://github.com/user-attachments/assets/878e293b-878e-46af-b9c4-e4c186552451" />
</div>

<br />

## What is this?
Picture this: you're chatting with your friends, when all of a sudden, you've thought of the perfect meme for the occasion! Quickly, you search your downloads folder, only to realize that you've forgotten the filename! (or worse, that the meme is on your other computer!)

memeity aims to fix this issue by hosting all of your memes on Google Drive - syncing to all of your devices. It also boasts a wide variety of features in order to make finding the perfect one easier.

## Features
- **Filename and OCR search**:  find your memes however you remember them - or however you don't.
- **Tags**: color-coded tags that sync everywhere. Select a bunch of memes and tag them at once, then search with `#tag`.
- **Easy setup with Google Drive**: just log into memeity with Google Drive for a near-instant setup. For your privacy, memeity only sees the files it creates itself and uses only a single folder.
- **Quickly add memes**: drag and drop, or paste your memes into the pane to instantly process new images!
- **GIF search**: not in your library yet? Type `!gif` before your search (or press `Ctrl+G`, or `Enter` when nothing matches) to search Tenor, then paste, copy or save the GIF straight from the pane.
- **Quick & powerful built-in editor**: edit and markup your memes & templates in an instant, right inside the memeity pane.
- **Automatic updates**: memeity handles all of its own updates, so you'll never have to worry about keeping up to date again.
- **Cross-platform**: memeity runs on Windows, macOS and Linux (x86-64 and ARM64), with a web-based client planned.
- **A quick tour**: new installs get a short walkthrough of the pane. Replay it any time from Settings > About.
- **No telemetry**: memeity never phones home - I don't need your data, silly! (The only exception is GIF search: when you search Tenor, your search text goes to Tenor (Google) - and only then.)

## Installation
Grab the [latest release](https://github.com/purestellenium/memeity/releases/latest) for your platform:

- **Windows**: run the `x64-setup.exe` (or `arm64-setup.exe` on ARM) and follow the wizard.
- **macOS**: open the `.dmg` and drag memeity into Applications.
- **Linux**: download the `.AppImage`, make it executable and run it. It updates itself. Or install the `.deb` (Debian, Ubuntu) or `.rpm` (Fedora, openSUSE); those open the release page when an update is ready.

The Linux builds need Ubuntu 24.04, Fedora 39, Debian 13 or newer (or any distro with a recent enough glibc). On Arch and similar, use the AppImage (it needs `fuse2`).

### Linux on Wayland
Wayland doesn't let apps set global shortcuts or type into other apps, so on Wayland memeity runs through XWayland and behaves a little differently:
- Bind your own shortcut: in your desktop's keyboard settings, add a custom shortcut that runs memeity (Settings > General shows the exact command). Pressing it again closes the pane.
- "Paste to App" copies instead: paste with `Ctrl+V`.

It works as usual on X11.

## Acknowledgements
Thanks to Ivie for helping me test it when it was still just in v0!

## AI Declaration
I used AI heavily (Claude Opus 5.5) in the development of memeity- I just wanted something that worked, and so using AI was the quickest way to make this happen.
The development process of this app was heavily supervised by me to ensure high code quality, but Rust is not a language I consider myself proficient in.

## Contributing
As always, PRs welcome!
