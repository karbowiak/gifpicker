<script lang="ts">
  import { onMount } from "svelte";
  import { searchSuggestions, performSearch, searchQuery, viewMode } from "$lib/stores/search";
  import { selectedIndex } from "$lib/stores/ui";

  export let onSelect: (suggestion: string) => void = () => {};

  let containerElement: HTMLDivElement;
  let contentElement: HTMLDivElement;
  let shouldAnimate = false;
  let maxScroll = 0;

  // Check if content overflows and needs animation
  function checkOverflow() {
    if (containerElement && contentElement) {
      const containerWidth = containerElement.offsetWidth;
      const contentWidth = contentElement.scrollWidth;
      shouldAnimate = contentWidth > containerWidth;
      maxScroll = contentWidth - containerWidth;
    }
  }

  // Recheck on suggestions change
  $: if ($searchSuggestions) {
    setTimeout(checkOverflow, 10);
  }

  onMount(() => {
    checkOverflow();
    window.addEventListener('resize', checkOverflow);
    return () => window.removeEventListener('resize', checkOverflow);
  });

  function handleClick(suggestion: string) {
    searchQuery.set(suggestion);
    viewMode.set('search');
    performSearch(suggestion);
    selectedIndex.set(-1);
    onSelect(suggestion);
  }
</script>

{#if $searchSuggestions.length > 0}
  <div class="suggestions-ticker" bind:this={containerElement}>
    <div 
      class="ticker-content" 
      class:animate={shouldAnimate}
      class:centered={!shouldAnimate}
      bind:this={contentElement}
      style={shouldAnimate ? `--max-scroll: -${maxScroll}px` : ''}
    >
      {#each $searchSuggestions as suggestion}
        <button class="suggestion-chip" on:click={() => handleClick(suggestion)}>
          {suggestion}
        </button>
      {/each}
    </div>
  </div>
{/if}

<style>
  .suggestions-ticker {
    width: 100%;
    overflow: hidden;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    padding: 8px 12px;
  }

  .ticker-content {
    display: flex;
    gap: 8px;
    width: max-content;
  }

  .ticker-content.centered {
    width: 100%;
    justify-content: center;
  }

  .ticker-content.animate {
    animation: bounce 16s ease-in-out infinite;
  }

  .suggestions-ticker:hover .ticker-content.animate {
    animation-play-state: paused;
  }

  @keyframes bounce {
    0%, 10% {
      transform: translateX(0);
    }
    45%, 55% {
      transform: translateX(var(--max-scroll));
    }
    90%, 100% {
      transform: translateX(0);
    }
  }

  .suggestion-chip {
    flex-shrink: 0;
    padding: 6px 14px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 16px;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    white-space: nowrap;
  }

  .suggestion-chip:hover {
    background: var(--accent-color);
    border-color: var(--accent-color);
    color: white;
    transform: scale(1.05);
  }
</style>
