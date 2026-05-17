<script lang="ts">
  import { recentSearches } from "$lib/stores/recent";
  import { favorites } from "$lib/stores/favorites";
  import type { Favorite } from "$lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";

  export let onSearchChip: (query: string) => void = () => {};
  export let onFavoriteChip: (favorite: Favorite) => void = () => {};

  // Top six most-used favorites, sorted by use_count desc then last_used desc.
  // Cheap derived value — the favorites list is tiny.
  $: topFavorites = [...$favorites]
    .filter((f) => f.use_count > 0)
    .sort((a, b) => {
      if (b.use_count !== a.use_count) return b.use_count - a.use_count;
      return (b.last_used ?? "").localeCompare(a.last_used ?? "");
    })
    .slice(0, 6);

  function thumbUrl(fav: Favorite): string | null {
    if (fav.mp4_filepath) return convertFileSrc(fav.mp4_filepath);
    if (fav.filepath) return convertFileSrc(fav.filepath);
    return fav.gif_url ?? null;
  }

  function isVideo(fav: Favorite): boolean {
    return !!fav.mp4_filepath;
  }
</script>

{#if $recentSearches.length > 0 || topFavorites.length > 0}
  <div class="chip-rail">
    {#if topFavorites.length > 0}
      <div class="chip-row">
        <span class="chip-label">Most used</span>
        <div class="chip-strip">
          {#each topFavorites as fav (fav.id)}
            {@const url = thumbUrl(fav)}
            <button
              class="thumb-chip"
              on:click={() => onFavoriteChip(fav)}
              title={fav.description || fav.filename}
              aria-label={`Copy ${fav.description || fav.filename}`}
            >
              {#if url}
                {#if isVideo(fav)}
                  <video src={url} muted playsinline autoplay loop></video>
                {:else}
                  <img src={url} alt="" loading="lazy" />
                {/if}
              {/if}
            </button>
          {/each}
        </div>
      </div>
    {/if}

    {#if $recentSearches.length > 0}
      <div class="chip-row">
        <span class="chip-label">Recent</span>
        <div class="chip-strip">
          {#each $recentSearches as query (query)}
            <button
              class="text-chip"
              on:click={() => onSearchChip(query)}
              title={`Search ${query}`}
            >
              {query}
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .chip-rail {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 8px 10px 4px;
  }

  .chip-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .chip-label {
    flex-shrink: 0;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: var(--text-tertiary);
    width: 64px;
  }

  .chip-strip {
    display: flex;
    gap: 6px;
    overflow-x: auto;
    overflow-y: hidden;
    flex: 1;
    /* Slim scrollbar that only appears on hover */
    scrollbar-width: thin;
    scrollbar-color: var(--border-color) transparent;
  }

  .chip-strip::-webkit-scrollbar {
    height: 4px;
  }
  .chip-strip::-webkit-scrollbar-track {
    background: transparent;
  }
  .chip-strip::-webkit-scrollbar-thumb {
    background: var(--border-color);
    border-radius: 2px;
  }

  .text-chip {
    flex-shrink: 0;
    padding: 5px 12px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 14px;
    color: var(--text-primary);
    font-size: 12px;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .text-chip:hover {
    background: var(--accent-color);
    border-color: var(--accent-color);
    color: white;
  }

  .thumb-chip {
    flex-shrink: 0;
    width: 44px;
    height: 44px;
    padding: 0;
    border: 1px solid var(--border-color);
    border-radius: 8px;
    overflow: hidden;
    background: var(--bg-secondary);
    cursor: pointer;
    transition: transform 0.15s ease, border-color 0.15s ease;
  }

  .thumb-chip:hover {
    transform: scale(1.06);
    border-color: var(--accent-color);
  }

  .thumb-chip img,
  .thumb-chip video {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
</style>
