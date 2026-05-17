<script lang="ts">
  import type { KlipyCategory } from "$lib/types";
  import { onMount, onDestroy } from "svelte";

  export let category: KlipyCategory;
  // index is forwarded by the parent for keying; not read here
  export const index: number = 0;
  export let selected: boolean = false;
  export let onClick: (category: KlipyCategory) => void = () => {};
  export let onHover: () => void = () => {};
  export let onLeave: () => void = () => {};

  let containerElement: HTMLDivElement;
  let videoElement: HTMLVideoElement;
  let observer: IntersectionObserver;

  onMount(() => {
    const scrollContainer = containerElement?.closest(".masonry-layout, .categories-container");

    observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
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
  on:click={handleClick}
  on:mouseenter={onHover}
  on:mouseleave={onLeave}
  role="button"
  tabindex="0"
  on:keydown={(e) => e.key === "Enter" && handleClick()}
>
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

  <!-- Uniform dark scrim so the centered title is legible regardless of the
       underlying gif's brightness. Mirrors Discord's category-picker style. -->
  <div class="scrim"></div>

  <span class="category-name">{category.name}</span>
</div>

<style>
  .category-item {
    position: relative;
    aspect-ratio: 16 / 10;
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

  .media {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .scrim {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    transition: background 0.15s ease;
  }

  .category-item:hover .scrim {
    background: rgba(0, 0, 0, 0.25);
  }

  .category-name {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 8px;
    text-align: center;
    font-size: 15px;
    font-weight: 700;
    color: white;
    text-shadow: 0 2px 6px rgba(0, 0, 0, 0.6);
    text-transform: lowercase;
    pointer-events: none;
  }
</style>
