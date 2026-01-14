<script lang="ts">
  import type { KlipyCategory } from "$lib/types";
  import { onMount, onDestroy } from "svelte";

  export let category: KlipyCategory;
  export let index: number = 0;
  export let selected: boolean = false;
  export let onClick: (category: KlipyCategory) => void = () => {};
  export let onHover: () => void = () => {};
  export let onLeave: () => void = () => {};

  let containerElement: HTMLDivElement;
  let videoElement: HTMLVideoElement;
  let observer: IntersectionObserver;
  let isVisible = false;

  // Grid row span - categories are square-ish
  const ROW_HEIGHT = 10;
  $: aspectRatio = category.width && category.height ? category.width / category.height : 1;
  $: rowSpan = Math.ceil((150 / aspectRatio) / ROW_HEIGHT) + 1;

  onMount(() => {
    const scrollContainer = containerElement?.closest('.masonry-layout');
    
    observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          isVisible = entry.isIntersecting;
          
          if (videoElement) {
            if (entry.isIntersecting) {
              videoElement.play().catch(() => {});
            } else {
              videoElement.pause();
            }
          }
        });
      },
      { root: scrollContainer, rootMargin: "100px", threshold: 0.01 },
    );
    if (containerElement) observer.observe(containerElement);
  });

  onDestroy(() => {
    if (observer && containerElement) {
      observer.unobserve(containerElement);
      observer.disconnect();
    }
  });

  function handleClick() {
    onClick(category);
  }
</script>

<div
  bind:this={containerElement}
  class="category-item"
  class:selected
  style="grid-row: span {rowSpan};"
  on:click={handleClick}
  on:mouseenter={onHover}
  on:mouseleave={onLeave}
  role="button"
  tabindex="0"
  on:keydown={(e) => e.key === "Enter" && handleClick()}
>
  <div class="media-wrapper">
    {#if category.mp4_url}
      <video
        bind:this={videoElement}
        src={category.mp4_url}
        class="media"
        autoplay
        loop
        muted
        playsinline
      ></video>
    {:else}
      <img
        src={category.gif_url}
        alt={category.name}
        class="media"
      />
    {/if}

    <div class="overlay">
      <span class="category-name">{category.name}</span>
    </div>
  </div>
</div>

<style>
  .category-item {
    position: relative;
    cursor: pointer;
    border-radius: 8px;
    overflow: hidden;
    background: var(--bg-secondary);
    transition: box-shadow 0.15s ease, transform 0.15s ease, z-index 0s;
  }

  .category-item:hover {
    box-shadow: 0 0 0 2px var(--accent-color);
  }

  .category-item.selected {
    box-shadow: 0 0 0 3px var(--accent-color), 0 8px 24px rgba(0, 0, 0, 0.4);
    transform: scale(1.05);
    z-index: 10;
  }

  .media-wrapper {
    position: relative;
    width: 100%;
    height: 100%;
  }

  .media {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .overlay {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      to top,
      rgba(0, 0, 0, 0.8) 0%,
      rgba(0, 0, 0, 0.2) 50%,
      transparent 100%
    );
    display: flex;
    align-items: flex-end;
    justify-content: center;
    padding: 12px;
  }

  .category-name {
    font-size: 14px;
    font-weight: 600;
    color: white;
    text-align: center;
    text-shadow: 0 1px 3px rgba(0, 0, 0, 0.5);
  }
</style>
