<script lang="ts">
  import type { KlipyAdResult } from "$lib/types";

  export let ad: KlipyAdResult;

  // Klipy ads come as full HTML documents that handle their own click +
  // impression tracking. We render them in a sandboxed iframe so the ad
  // script can't reach into the rest of the app.
  //
  // sandbox flags:
  //   allow-scripts          — Klipy's content includes a small impression script
  //   allow-popups           — link clicks open the advertiser in a new tab/window
  //   allow-popups-to-escape-sandbox — the new window isn't itself sandboxed
  //
  // We deliberately do NOT grant: allow-same-origin (would let the ad read
  // app state from the parent document), allow-forms, allow-modals.
  const SANDBOX =
    "allow-scripts allow-popups allow-popups-to-escape-sandbox";

  // Compute how many masonry rows this ad spans. Mirrors the calculation in
  // MediaItem.svelte so ads sit cleanly in the grid alongside GIFs.
  const ROW_HEIGHT = 10;
  $: aspectRatio = ad.height > 0 ? ad.width / ad.height : 16 / 9;
  $: rowSpan = Math.ceil((150 / aspectRatio) / ROW_HEIGHT) + 1;
</script>

<div
  class="ad-item"
  style="grid-row: span {rowSpan}; aspect-ratio: {ad.width} / {ad.height};"
  role="complementary"
  aria-label="Sponsored content"
>
  <iframe
    title="Sponsored"
    srcdoc={ad.content}
    sandbox={SANDBOX}
    referrerpolicy="no-referrer-when-downgrade"
    loading="lazy"
  ></iframe>
  <span class="ad-badge">Ad</span>
</div>

<style>
  .ad-item {
    position: relative;
    overflow: hidden;
    border-radius: 4px;
    background: var(--bg-secondary);
    /* Subtle border so users can tell ads apart from GIFs at a glance */
    outline: 1px dashed var(--border-color);
    outline-offset: -1px;
  }

  iframe {
    display: block;
    width: 100%;
    height: 100%;
    border: 0;
    /* Block any cursor interaction from leaking into our grid hover handlers */
    background: transparent;
  }

  .ad-badge {
    position: absolute;
    top: 4px;
    left: 4px;
    padding: 2px 6px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.5px;
    text-transform: uppercase;
    color: white;
    background: rgba(0, 0, 0, 0.65);
    border-radius: 3px;
    pointer-events: none;
    z-index: 1;
  }
</style>
