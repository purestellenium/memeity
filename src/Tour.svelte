<script lang="ts">
  // First-run tour of the pane: a spotlight moves between the real controls (marked with
  // data-tour="…" in App.svelte) while a small card explains each one.
  import { onMount, tick } from "svelte";
  import { fade } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  let {
    shortcut,
    pasteFirst,
    canPaste = true,
    onDone,
  }: {
    /** The shortcut that opens the pane, e.g. "Ctrl+Alt+M". */
    shortcut: string;
    /** Enter pastes (true) or copies (false); see Settings > General. */
    pasteFirst: boolean;
    /** False on Linux Wayland, where memeity can only copy. */
    canPaste?: boolean;
    onDone: () => void;
  } = $props();

  const isMac = navigator.userAgent.includes("Mac");
  const MOD = isMac ? "⌘" : "Ctrl";
  const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  type Step = {
    /** The data-tour="…" element to spotlight; null centres the card with no spotlight. */
    target: string | null;
    title: string;
    body: string;
    /** Key caps shown under the text, each group one shortcut. */
    keys?: { caps: string[]; label: string }[];
  };

  const steps: Step[] = $derived([
    {
      target: null,
      title: "Welcome to memeity",
      body: "Your memes, searchable and one keypress away from any chat. Here's a quick look around.",
    },
    {
      target: "search",
      title: "Search everything",
      body: "Type a meme's name, any words in the picture, or a #tag. Results update as you type.",
    },
    {
      target: "results",
      title: "Send it straight to the chat",
      body: `Pick a meme with the arrow keys or the mouse. ${pasteFirst ? "Enter pastes it into the app you were just in" : "Enter copies it to your clipboard"}.`,
      keys: [
        { caps: ["↵"], label: pasteFirst ? "Paste" : "Copy" },
        ...(canPaste ? [{ caps: ["Shift", "↵"], label: pasteFirst ? "Copy" : "Paste" }] : []),
        { caps: [MOD, "1–9"], label: "Quick pick" },
      ],
    },
    {
      target: "footer",
      title: "Do more with a meme",
      body: "Rename it, tag it, pin it to the top or edit it. Right-click a meme for the same menu.",
      keys: [
        { caps: [MOD, "K"], label: "Actions" },
        { caps: [MOD, "T"], label: "Tags" },
        { caps: [MOD, "E"], label: "Edit" },
      ],
    },
    {
      target: "folder",
      title: "Add your own",
      body: "Drag images or links onto this window, or paste them. They go in your memeity folder on Google Drive and sync to every device.",
      keys: [{ caps: [MOD, "V"], label: "Paste an image" }],
    },
    {
      target: "search",
      title: "Not in your library yet?",
      body: "Start a search with !gif to look on Tenor instead. Paste a GIF straight away, or save it for next time.",
      keys: [
        { caps: [MOD, "G"], label: "Search Tenor" },
        { caps: [MOD, "S"], label: "Save a GIF" },
      ],
    },
    {
      target: "settings",
      title: "You're all set",
      // Linux Wayland (no pasting there either): the shortcut is set in the desktop's own
      // settings, which Settings > General explains.
      body: canPaste
        ? "Open memeity from anywhere with your shortcut. Themes, the shortcut itself and what Enter does are in Settings."
        : "Settings > General shows how to open memeity with a shortcut from anywhere. Themes and more are there too.",
      keys: [
        ...(canPaste ? [{ caps: shortcut.split("+"), label: "Open memeity" }] : []),
        { caps: [MOD, "/"], label: "All shortcuts" },
      ],
    },
  ]);

  let index = $state(0);
  const step = $derived(steps[index]);
  const last = $derived(index === steps.length - 1);

  // ---------- placement ----------

  /** Space around a spotlighted control, and between it and the card. */
  const PAD = 6;
  const GAP = 12;
  const EDGE = 12;

  type Box = { x: number; y: number; w: number; h: number; r: number };
  let spot = $state<Box | null>(null);
  let card: HTMLDivElement | undefined = $state();
  let cardPos = $state({ x: 0, y: 0 });
  /** Placed at least once: before that the card appears without sliding in from 0,0. */
  let placed = $state(false);

  function measure() {
    const el = step.target ? document.querySelector<HTMLElement>(`[data-tour="${step.target}"]`) : null;
    const W = window.innerWidth;
    const H = window.innerHeight;
    if (el) {
      const r = el.getBoundingClientRect();
      // Keep the spotlight inside the window (the results area runs edge to edge).
      const x = Math.max(PAD, r.left - PAD);
      const y = Math.max(PAD, r.top - PAD);
      spot = {
        x,
        y,
        w: Math.min(W - PAD, r.right + PAD) - x,
        h: Math.min(H - PAD, r.bottom + PAD) - y,
        r: Math.min(12, parseFloat(getComputedStyle(el).borderRadius) + PAD || 10),
      };
    } else {
      spot = null;
    }

    const cw = card?.offsetWidth ?? 320;
    const ch = card?.offsetHeight ?? 180;
    let x = (W - cw) / 2;
    let y = (H - ch) / 2;
    if (spot) {
      const below = spot.y + spot.h + GAP;
      const above = spot.y - GAP - ch;
      if (below + ch <= H - EDGE) y = below;
      else if (above >= EDGE) y = above;
      else y = spot.y + spot.h - ch - GAP * 2; // big target: sit inside it, near the bottom
      x = spot.x + spot.w / 2 - cw / 2;
    }
    cardPos = {
      x: Math.round(Math.min(Math.max(EDGE, x), W - cw - EDGE)),
      y: Math.round(Math.min(Math.max(EDGE, y), H - ch - EDGE)),
    };
  }

  // Re-place on every step (after the card's new text has laid out).
  $effect(() => {
    void index;
    tick().then(() => {
      measure();
      requestAnimationFrame(() => (placed = true));
    });
  });

  onMount(() => {
    const observer = new ResizeObserver(() => measure());
    observer.observe(document.body);
    if (card) observer.observe(card);
    window.addEventListener("resize", measure);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", measure);
    };
  });

  // ---------- navigation ----------

  function next() {
    if (last) onDone();
    else index++;
  }

  function back() {
    if (index > 0) index--;
  }

  /** Capture phase: the pane's own shortcuts never fire under the tour. */
  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Tab") return; // move between the card's buttons
    e.stopImmediatePropagation();
    if (e.ctrlKey || e.metaKey || e.altKey) {
      e.preventDefault();
      return;
    }
    switch (e.key) {
      case "Enter":
      case "ArrowRight":
        // Enter on a focused button is that button's click.
        if (e.key === "Enter" && document.activeElement instanceof HTMLButtonElement && card?.contains(document.activeElement)) return;
        e.preventDefault();
        next();
        break;
      case "ArrowLeft":
        e.preventDefault();
        back();
        break;
      case "Escape":
        e.preventDefault();
        onDone();
        break;
      default:
        e.preventDefault();
    }
  }

  const fadeIn = { duration: reduceMotion ? 0 : 220, easing: cubicOut };
</script>

<svelte:window onkeydowncapture={onKeydown} />

<div class="tour" transition:fade={fadeIn} role="dialog" aria-modal="true" aria-labelledby="tour-title" aria-describedby="tour-body">
  <!-- The dimmed page, with a hole cut around the current control. -->
  <div
    class="spot"
    class:hidden={!spot}
    class:placed
    style:left="{spot ? spot.x : cardPos.x + (card?.offsetWidth ?? 0) / 2}px"
    style:top="{spot ? spot.y : cardPos.y + (card?.offsetHeight ?? 0) / 2}px"
    style:width="{spot?.w ?? 0}px"
    style:height="{spot?.h ?? 0}px"
    style:border-radius="{spot?.r ?? 0}px"
    aria-hidden="true"
  ></div>

  <div class="card" class:placed bind:this={card} style:transform="translate({cardPos.x}px, {cardPos.y}px)">
    <div class="progress" aria-hidden="true">
      <i style:width="{((index + 1) / steps.length) * 100}%"></i>
    </div>
    {#key index}
      <div class="content" in:fade={{ duration: reduceMotion ? 0 : 200, delay: reduceMotion ? 0 : 60 }}>
        <p class="count">{index + 1} of {steps.length}</p>
        <h2 id="tour-title">{step.title}</h2>
        <p id="tour-body">{step.body}</p>
        {#if step.keys}
          <ul class="keys">
            {#each step.keys as k (k.label)}
              <li>
                <span class="caps">{#each k.caps as c, i (i)}<kbd>{c}</kbd>{/each}</span>
                {k.label}
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    {/key}
    <div class="actions">
      {#if !last}
        <button class="btn ghost" onclick={onDone}>Skip</button>
      {/if}
      <span class="spacer"></span>
      {#if index > 0}
        <button class="btn ghost" onclick={back} aria-label="Back">Back</button>
      {/if}
      <button class="btn primary" onclick={next}>
        {last ? "Start using memeity" : index === 0 ? "Show me" : "Next"}
        <kbd>↵</kbd>
      </button>
    </div>
  </div>
</div>

<style>
  .tour {
    position: fixed;
    inset: 0;
    z-index: 100;
  }

  /* One easing for everything that moves, so the spotlight and card travel together. */
  .tour {
    --ease: cubic-bezier(0.32, 0.72, 0, 1);
    --move: 480ms;
  }

  .spot {
    position: absolute;
    pointer-events: none;
    box-shadow:
      0 0 0 1px color-mix(in srgb, var(--fg) 22%, transparent),
      0 0 0 200vmax rgb(0 0 0 / 0.6);
  }

  .spot.placed {
    transition:
      left var(--move) var(--ease),
      top var(--move) var(--ease),
      width var(--move) var(--ease),
      height var(--move) var(--ease),
      border-radius var(--move) var(--ease),
      box-shadow 200ms ease;
  }

  /* No control to show: the whole pane stays dimmed. */
  .spot.hidden {
    box-shadow: 0 0 0 200vmax rgb(0 0 0 / 0.6);
  }

  .card {
    position: absolute;
    top: 0;
    left: 0;
    width: min(320px, calc(100vw - 24px));
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: 12px;
    background: var(--popover);
    box-shadow:
      0 1px 2px rgb(0 0 0 / 0.2),
      0 12px 32px -8px rgb(0 0 0 / 0.45);
  }

  .card.placed {
    transition: transform var(--move) var(--ease);
  }

  .progress {
    height: 2px;
    background: var(--muted);
  }

  .progress i {
    display: block;
    height: 100%;
    background: var(--fg);
    transition: width var(--move) var(--ease);
  }

  .content {
    padding: 14px 16px 4px;
  }

  .count {
    margin: 0 0 4px;
    color: var(--muted-fg);
    font: 500 11px/1 "Geist Mono Variable", ui-monospace, "Cascadia Mono", monospace;
    letter-spacing: 0.02em;
  }

  h2 {
    margin: 0 0 6px;
    font-size: 15px;
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  .content p:not(.count) {
    margin: 0;
    color: var(--muted-fg);
    font-size: 13px;
  }

  .keys {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 14px;
    margin: 12px 0 0;
    padding: 0;
    list-style: none;
    color: var(--muted-fg);
    font-size: 12px;
  }

  .keys li {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .caps {
    display: inline-flex;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 12px 12px 12px 16px;
  }

  .spacer {
    flex: 1;
  }

  .btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 12px;
    border: 1px solid transparent;
    border-radius: 7px;
    background: none;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: background 120ms ease, color 120ms ease;
  }

  .btn.ghost {
    color: var(--muted-fg);
  }

  .btn.ghost:hover {
    background: var(--accent);
    color: var(--fg);
  }

  /* "Skip" sits flush with the text above it. */
  .actions > .btn.ghost:first-child {
    margin-left: -12px;
  }

  .btn.primary {
    background: var(--primary);
    color: var(--primary-fg);
  }

  .btn.primary:hover {
    background: color-mix(in srgb, var(--primary) 88%, transparent);
  }

  .btn.primary kbd {
    border-color: color-mix(in srgb, var(--primary-fg) 20%, transparent);
    background: color-mix(in srgb, var(--primary-fg) 10%, transparent);
    color: inherit;
    opacity: 0.75;
  }
</style>
