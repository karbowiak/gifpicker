<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { Favorite, KlipyResultItem, KlipyGifResult, TileSize } from "$lib/types";
  import { isKlipyAd } from "$lib/types";
  import MediaItem from "./MediaItem.svelte";
  import AdItem from "./AdItem.svelte";
  import { selectedIndex } from "$lib/stores/ui";
  import { settings } from "$lib/stores/settings";

  // Mixed grid: local favorites, klipy gifs, and klipy ads. Ads render via
  // AdItem; everything else via MediaItem.
  export let items: (Favorite | KlipyResultItem)[] = [];
  export let onItemClick: (item: Favorite | KlipyGifResult) => void = () => {};
  export let onScrollNearEnd: (() => void) | undefined = undefined;
  export let showSkeleton: boolean = false;

  // Column counts per tile size — bumps by ±1 from the medium default at each
  // breakpoint. The masonry grid then auto-fills based on these widths.
  const SIZE_COLS: Record<TileSize, [number, number, number]> = {
    small:  [6, 5, 3],   // 6 desktop / 5 tablet / 3 narrow
    medium: [4, 3, 2],
    large:  [3, 2, 2],
  };

  $: tileSize = ($settings?.tile_size ?? 'medium') as TileSize;
  $: [colsLarge, colsMid, colsSmall] = SIZE_COLS[tileSize];

  let containerElement: HTMLDivElement;
  let sentinelElement: HTMLDivElement;
  let previousSelectedIndex = -1;
  let scrollObserver: IntersectionObserver | undefined;

  // Auto-scroll the selected item into view only when selection actually changes
  // (not on every items[] mutation — otherwise infinite-scroll appends would jump).
  $: if (
    $selectedIndex !== previousSelectedIndex &&
    $selectedIndex >= 0 &&
    $selectedIndex < items.length
  ) {
    scrollToSelectedItem();
    previousSelectedIndex = $selectedIndex;
  }

  function scrollToSelectedItem() {
    if (!containerElement) return;
    const selectedEl = containerElement.querySelector(".media-item.selected");
    if (selectedEl) {
      selectedEl.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  }

  // Set up infinite scroll
  $: if (onScrollNearEnd && sentinelElement) {
    setupInfiniteScroll();
  }

  // Re-check infinite scroll when items change (in case we didn't fill the screen)
  $: if (items && scrollObserver && sentinelElement) {
    // Small delay to allow DOM to update
    setTimeout(() => {
      if (scrollObserver && sentinelElement) {
        scrollObserver.unobserve(sentinelElement);
        scrollObserver.observe(sentinelElement);
      }
    }, 100);
  }

  function setupInfiniteScroll() {
    if (scrollObserver) scrollObserver.disconnect();
    if (!sentinelElement || !onScrollNearEnd) return;

    scrollObserver = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            if (onScrollNearEnd) {
              onScrollNearEnd();
            }
          }
        });
      },
      { root: containerElement, rootMargin: "400px", threshold: 0.01 },
    );
    scrollObserver.observe(sentinelElement);
  }

  onMount(() => {
    if ($selectedIndex >= 0 && $selectedIndex < items.length) {
      scrollToSelectedItem();
    }
  });

  onDestroy(() => {
    if (scrollObserver) scrollObserver.disconnect();
  });
</script>

<div
  class="masonry-layout"
  class:size-small={tileSize === 'small'}
  class:size-large={tileSize === 'large'}
  bind:this={containerElement}
>
  {#if showSkeleton && items.length === 0}
    <div class="masonry-grid">
      {#each Array(18) as _, i (i)}
        {@const span = 12 + ((i * 7) % 16)}
        <div class="skeleton-tile" style="grid-row: span {span};"></div>
      {/each}
    </div>
  {:else if items.length === 0}
    <div class="empty-state">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none">
        <path
          d="M21 19V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2zM8.5 13.5l2.5 3.01L14.5 12l4.5 6H5l3.5-4.5z"
          fill="currentColor"
          opacity="0.4"
        />
      </svg>
      <p>Search for GIFs to get started</p>
    </div>
  {:else}
    <div class="masonry-grid">
      {#each items as item, index (
        isKlipyAd(item)
          ? `ad:${index}:${item.content.length}`
          : (('id' in item && item.id) ?? `i:${index}`)
      )}
        {#if isKlipyAd(item)}
          <AdItem ad={item} />
        {:else}
          <MediaItem
            item={item as Favorite | KlipyGifResult}
            {index}
            selected={$selectedIndex === index}
            onClick={onItemClick}
            onHover={() => selectedIndex.set(index)}
            onLeave={() => selectedIndex.set(-1)}
          />
        {/if}
      {/each}
    </div>

    {#if onScrollNearEnd}
      <div class="scroll-sentinel" bind:this={sentinelElement}></div>
    {/if}
  {/if}
</div>

<style>
  .masonry-layout {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 8px;
    background: var(--bg-primary);
  }

  /* CSS Grid masonry - items span rows based on aspect ratio.
     Column count switches with .size-small / .size-large modifiers. */
  .masonry-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    grid-auto-rows: 10px;
    gap: 4px;
  }

  .masonry-layout.size-small .masonry-grid { grid-template-columns: repeat(6, 1fr); }
  .masonry-layout.size-large .masonry-grid { grid-template-columns: repeat(3, 1fr); }

  /* Responsive columns - default size */
  @media (max-width: 900px) {
    .masonry-grid { grid-template-columns: repeat(3, 1fr); }
    .masonry-layout.size-small .masonry-grid { grid-template-columns: repeat(5, 1fr); }
    .masonry-layout.size-large .masonry-grid { grid-template-columns: repeat(2, 1fr); }
  }

  @media (max-width: 600px) {
    .masonry-grid { grid-template-columns: repeat(2, 1fr); }
    .masonry-layout.size-small .masonry-grid { grid-template-columns: repeat(3, 1fr); }
    .masonry-layout.size-large .masonry-grid { grid-template-columns: repeat(2, 1fr); }
  }

  @media (max-width: 400px) {
    .masonry-grid,
    .masonry-layout.size-small .masonry-grid,
    .masonry-layout.size-large .masonry-grid { grid-template-columns: 1fr; }
  }

  .skeleton-tile {
    background: linear-gradient(
      90deg,
      var(--bg-secondary) 25%,
      var(--bg-tertiary) 50%,
      var(--bg-secondary) 75%
    );
    background-size: 200% 100%;
    animation: tile-shimmer 1.4s ease-in-out infinite;
    border-radius: 4px;
  }

  @keyframes tile-shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 20px;
    text-align: center;
    color: var(--text-tertiary);
    opacity: 0.6;
  }

  .empty-state svg {
    margin-bottom: 12px;
  }

  .empty-state p {
    font-size: 13px;
    margin: 0;
  }

  /* Minimal scrollbar */
  .masonry-layout::-webkit-scrollbar {
    width: 6px;
  }

  .masonry-layout::-webkit-scrollbar-track {
    background: transparent;
  }

  .masonry-layout::-webkit-scrollbar-thumb {
    background: var(--border-color);
    border-radius: 3px;
  }

  .masonry-layout::-webkit-scrollbar-thumb:hover {
    background: var(--text-tertiary);
  }

  .scroll-sentinel {
    width: 100%;
    height: 10px;
    pointer-events: none;
  }
</style>
