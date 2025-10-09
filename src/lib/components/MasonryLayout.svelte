<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { Favorite, GiphyGifResult } from '$lib/types';
  import MediaItem from './MediaItem.svelte';
  import { selectedIndex } from '$lib/stores/ui';

  export let items: (Favorite | GiphyGifResult)[] = [];
  export let onItemClick: (item: Favorite | GiphyGifResult) => void = () => {};
  export let onScrollNearEnd: (() => void) | undefined = undefined;

  let containerElement: HTMLDivElement;
  let sentinelElement: HTMLDivElement;
  let currentSelectedIndex = -1;
  let previousSelectedIndex = -1;
  let scrollObserver: IntersectionObserver | undefined;

  // Subscribe to selected index
  selectedIndex.subscribe(value => {
    currentSelectedIndex = value;
  });

  // Scroll to selected item ONLY when selection actually changes (not when items are added)
  $: if (currentSelectedIndex !== previousSelectedIndex && currentSelectedIndex >= 0 && currentSelectedIndex < items.length) {
    scrollToSelectedItem();
    previousSelectedIndex = currentSelectedIndex;
  }

  function scrollToSelectedItem() {
    if (!containerElement) return;

    const selectedEl = containerElement.querySelector('.media-item.selected');
    if (selectedEl) {
      selectedEl.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
  }

  // Set up infinite scroll when component mounts or callback changes
  $: if (onScrollNearEnd && sentinelElement) {
    setupInfiniteScroll();
  }

  function setupInfiniteScroll() {
    // Clean up existing observer
    if (scrollObserver) {
      scrollObserver.disconnect();
    }

    if (!sentinelElement || !onScrollNearEnd) return;

    scrollObserver = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting && onScrollNearEnd) {
            onScrollNearEnd();
          }
        });
      },
      {
        root: containerElement,
        rootMargin: '400px', // Trigger 400px before reaching the bottom
        threshold: 0.01
      }
    );

    scrollObserver.observe(sentinelElement);
  }

  onMount(() => {
    // Don't reset selection - let it persist across window open/close
    // Only scroll to the current selection if it exists
    if (currentSelectedIndex >= 0 && currentSelectedIndex < items.length) {
      scrollToSelectedItem();
    }
  });

  onDestroy(() => {
    if (scrollObserver) {
      scrollObserver.disconnect();
    }
  });
</script>

<div
  class="masonry-layout"
  bind:this={containerElement}
>
  {#if items.length === 0}
    <div class="empty-state">
      <svg width="64" height="64" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M21 19V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2zM8.5 13.5l2.5 3.01L14.5 12l4.5 6H5l3.5-4.5z" fill="currentColor" opacity="0.3"/>
      </svg>
      <h3>No GIFs yet</h3>
      <p>Search for GIFs to get started</p>
    </div>
  {:else}
    <div class="masonry-grid">
      {#each items as item, index (item.id || index)}
        <MediaItem
          {item}
          selected={currentSelectedIndex === index}
          onClick={onItemClick}
        />
      {/each}
    </div>

    {#if onScrollNearEnd}
      <!-- Sentinel element for infinite scroll detection -->
      <div class="scroll-sentinel" bind:this={sentinelElement}></div>
    {/if}
  {/if}
</div>

<style>
  .masonry-layout {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 16px;
  }

  .masonry-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 8px;
    align-items: start;
  }

  /* Responsive column counts */
  @media (max-width: 1200px) {
    .masonry-grid {
      grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
      gap: 8px;
    }
  }

  @media (max-width: 800px) {
    .masonry-grid {
      grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
      gap: 6px;
    }
  }

  @media (max-width: 500px) {
    .masonry-grid {
      grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
      gap: 6px;
    }
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 80px 20px;
    text-align: center;
    color: var(--text-tertiary, #9ca3af);
  }

  .empty-state svg {
    margin-bottom: 16px;
  }

  .empty-state h3 {
    font-size: 18px;
    font-weight: 600;
    margin: 0 0 8px 0;
    color: var(--text-secondary, #6b7280);
  }

  .empty-state p {
    font-size: 14px;
    margin: 0;
  }

  /* Custom scrollbar */
  .masonry-layout::-webkit-scrollbar {
    width: 8px;
  }

  .masonry-layout::-webkit-scrollbar-track {
    background: var(--bg-secondary, #f9fafb);
  }

  .masonry-layout::-webkit-scrollbar-thumb {
    background: var(--border-color, #e5e7eb);
    border-radius: 4px;
  }

  .masonry-layout::-webkit-scrollbar-thumb:hover {
    background: var(--text-tertiary, #9ca3af);
  }

  .scroll-sentinel {
    width: 100%;
    height: 20px;
    margin-top: 16px;
    pointer-events: none;
  }
</style>
