<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { Favorite, KlipyGifResult } from "$lib/types";
  import MediaItem from "./MediaItem.svelte";
  import { selectedIndex } from "$lib/stores/ui";

  export let items: (Favorite | KlipyGifResult)[] = [];
  export let onItemClick: (item: Favorite | KlipyGifResult) => void = () => {};
  export let onScrollNearEnd: (() => void) | undefined = undefined;

  let containerElement: HTMLDivElement;
  let sentinelElement: HTMLDivElement;
  let currentSelectedIndex = -1;
  let previousSelectedIndex = -1;
  let scrollObserver: IntersectionObserver | undefined;

  // Subscribe to selected index
  selectedIndex.subscribe((value) => {
    currentSelectedIndex = value;
  });

  // Scroll to selected item ONLY when selection actually changes
  $: if (
    currentSelectedIndex !== previousSelectedIndex &&
    currentSelectedIndex >= 0 &&
    currentSelectedIndex < items.length
  ) {
    scrollToSelectedItem();
    previousSelectedIndex = currentSelectedIndex;
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
    if (currentSelectedIndex >= 0 && currentSelectedIndex < items.length) {
      scrollToSelectedItem();
    }
  });

  onDestroy(() => {
    if (scrollObserver) scrollObserver.disconnect();
  });
</script>

<div class="masonry-layout" bind:this={containerElement}>
  {#if items.length === 0}
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
      {#each items.filter((i) => !!i) as item, index (item.id || index)}
        <MediaItem
          {item}
          {index}
          selected={currentSelectedIndex === index}
          onClick={onItemClick}
          onHover={() => selectedIndex.set(index)}
          onLeave={() => selectedIndex.set(-1)}
        />
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

  /* CSS Grid masonry - items span rows based on aspect ratio */
  .masonry-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    grid-auto-rows: 10px;
    gap: 4px;
  }

  /* Responsive columns */
  @media (max-width: 900px) {
    .masonry-grid {
      grid-template-columns: repeat(3, 1fr);
    }
  }

  @media (max-width: 600px) {
    .masonry-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  @media (max-width: 400px) {
    .masonry-grid {
      grid-template-columns: 1fr;
    }
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
