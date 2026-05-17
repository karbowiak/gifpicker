<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import SearchBar from "$lib/components/SearchBar.svelte";
  import MasonryLayout from "$lib/components/MasonryLayout.svelte";
  import CategoryItem from "$lib/components/CategoryItem.svelte";
  import SearchSuggestionsTicker from "$lib/components/SearchSuggestionsTicker.svelte";
  import Toast from "$lib/components/Toast.svelte";
  import ContextMenu from "$lib/components/ContextMenu.svelte";
  import Settings from "$lib/components/Settings.svelte";
  import {
    searchResults,
    searchQuery,
    loadMoreResults,
    isLoadingMore,
    isSearching,
    clearSearch,
    performSearch,
    viewMode,
    categories,
    currentCategory,
    loadCategoryGifs,
    loadCategories,
    goHome,
    searchSuggestions,
    fetchSearchSuggestions,
    saveKlipyGif,
  } from "$lib/stores/search";
  import { favorites } from "$lib/stores/favorites";
  import { settings } from "$lib/stores/settings";
  import { selectedIndex, showToast, showSettings } from "$lib/stores/ui";
  import type {
    Favorite,
    KlipyGifResult,
    KlipyResultItem,
    KlipyCategory,
  } from "$lib/types";
  import { isFavorite, isKlipyAd } from "$lib/types";
  import {
    gridColumnsForWidth,
    keyToDirection,
    nextIndex,
  } from "$lib/utils/gridNavigation";
  import { copyItem } from "$lib/utils/copyMedia";

  let allItems: (Favorite | KlipyResultItem)[] = [];
  let isLoading = true;
  let searchBarComponent: SearchBar;

  // Flatten search results into a single ordered list for the grid + keyboard nav.
  // Includes ad items — the grid renders them, but the keyboard handler and
  // click handler skip past them so the user only ever "selects" real content.
  $: {
    const items: (Favorite | KlipyResultItem)[] = [];
    const previousLength = allItems.length;

    if ($viewMode === "favorites") {
      if ($searchResults.local?.length) items.push(...$searchResults.local);
    } else if (
      $viewMode === "trending" ||
      $viewMode === "search" ||
      $viewMode === "category"
    ) {
      if ($searchResults.klipy?.items?.length)
        items.push(...$searchResults.klipy.items);
    }
    // 'categories' view renders a separate grid below.

    allItems = items;

    // Don't clobber selection during infinite-scroll appends — only when the
    // selected index is genuinely off the end of a non-growing list.
    const isAddingMore = items.length > previousLength && previousLength > 0;
    if (!isAddingMore && $selectedIndex >= allItems.length) {
      selectedIndex.set(-1);
    }
  }

  // Selectable = non-ad. Keyboard nav and Enter only land on these.
  function isSelectable(item: Favorite | KlipyResultItem): boolean {
    return !isKlipyAd(item);
  }

  // Walk forward (or backward) from `from` to the nearest selectable index.
  function nextSelectable(from: number, step: 1 | -1): number {
    let i = from;
    while (i >= 0 && i < allItems.length) {
      if (isSelectable(allItems[i])) return i;
      i += step;
    }
    // Wrap around past either end so we don't strand the user on an ad row.
    if (step === 1) {
      for (let j = 0; j < allItems.length; j++)
        if (isSelectable(allItems[j])) return j;
    } else {
      for (let j = allItems.length - 1; j >= 0; j--)
        if (isSelectable(allItems[j])) return j;
    }
    return -1;
  }

  // Refresh related-search ticker whenever the query in search view changes.
  $: if ($viewMode === "search" && $searchQuery.trim()) {
    fetchSearchSuggestions($searchQuery);
  }

  async function handleScrollNearEnd() {
    if ($viewMode === "favorites" || $viewMode === "categories" || $isLoadingMore)
      return;
    await loadMoreResults();
  }

  async function copyItemToClipboard(item: Favorite | KlipyGifResult) {
    const mode = $settings?.clipboard_mode ?? "file";
    const format = $settings?.clipboard_format ?? "gif";
    const result = await copyItem(item, mode, format);

    if (!result.ok) {
      showToast(result.reason, "error");
      return;
    }

    if (isFavorite(item) && item.id) {
      await invoke("increment_use_count", { id: item.id });
    }
    showToast(
      result.via === "url" ? "URL copied to clipboard!" : "Copied to clipboard!",
      "success",
    );
  }

  async function handleItemClick(item: Favorite | KlipyResultItem) {
    if (isKlipyAd(item)) return; // ad clicks are handled inside the iframe
    try {
      await copyItemToClipboard(item as Favorite | KlipyGifResult);

      if ($settings?.close_after_selection ?? true) {
        // Reset state before closing so the next open lands on favorites.
        clearSearch();
        await performSearch("");
        try {
          await invoke("close_window");
        } catch (error) {
          console.error("Failed to close window:", error);
        }
      }
    } catch (error) {
      console.error("Failed to copy to clipboard:", error);
      showToast("Failed to copy to clipboard", "error");
    }
  }

  async function toggleFavorite() {
    const current = $selectedIndex;
    if (current < 0 || current >= allItems.length) return;
    const item = allItems[current];
    if (isKlipyAd(item)) return;

    if (isFavorite(item)) {
      if (!item.id) return;
      try {
        await favorites.delete(item.id);
        searchResults.update((r) => ({
          ...r,
          local: r.local.filter((f) => f.id !== item.id),
        }));
        showToast("Removed from favorites", "success");
      } catch {
        showToast("Failed to remove", "error");
      }
    } else {
      try {
        await saveKlipyGif(item as KlipyGifResult);
        showToast("Added to favorites!", "success");
      } catch {
        showToast("Failed to add", "error");
      }
    }
  }

  function handleCategoryClick(category: KlipyCategory) {
    loadCategoryGifs(category);
  }

  function focusGridFromSearchInput(input: HTMLInputElement) {
    input.blur();
    // First selectable item — skip an ad if it happens to be at index 0.
    const first = nextSelectable(0, 1);
    selectedIndex.set(first >= 0 ? first : 0);
    document
      .querySelector(".masonry-layout")
      ?.scrollTo({ top: 0, behavior: "smooth" });
  }

  function handleKeyDown(event: KeyboardEvent) {
    // When typing in the search box, only Up/Down should escape into the grid.
    if (event.target instanceof HTMLInputElement) {
      if (event.key === "ArrowDown" || event.key === "ArrowUp") {
        event.preventDefault();
        focusGridFromSearchInput(event.target);
      }
      return;
    }

    const itemCount = allItems.length;
    const current = $selectedIndex;

    const direction = keyToDirection(event.key);
    if (direction) {
      event.preventDefault();
      const cols = gridColumnsForWidth(window.innerWidth);
      const target = nextIndex(current, direction, itemCount, cols);
      if (target === null) return;
      // Skip ad slots so keyboard nav only lands on selectable content.
      const landing = isSelectable(allItems[target])
        ? target
        : nextSelectable(target, direction === "left" || direction === "up" ? -1 : 1);
      if (landing >= 0) selectedIndex.set(landing);
      return;
    }

    switch (event.key) {
      case "Enter":
        event.preventDefault();
        if (current >= 0 && current < itemCount && isSelectable(allItems[current]))
          handleItemClick(allItems[current]);
        break;
      case "f":
      case "F":
        event.preventDefault();
        toggleFavorite();
        break;
      case "Escape":
        event.preventDefault();
        clearSearch();
        performSearch("");
        isSearching.set(false);
        isLoadingMore.set(false);
        invoke("close_window").catch(console.error);
        break;
    }
  }

  onMount(async () => {
    try {
      await settings.load();
      await favorites.load();

      const allFavorites = await invoke<Favorite[]>("get_all_favorites");
      searchResults.set({ local: allFavorites, klipy: undefined });

      isLoading = false;

      await listen("open-settings", () => showSettings.set(true));
      await listen("focus-search", () => searchBarComponent?.focus());
      await listen("clear-search", () => {
        isSearching.set(false);
        isLoadingMore.set(false);
        searchBarComponent?.clear();
      });

      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const currentWindow = getCurrentWindow();

      await currentWindow.onFocusChanged(({ payload: focused }) => {
        if (!focused) return;
        // Reset everything to a known-clean state on each show.
        isSearching.set(false);
        isLoadingMore.set(false);
        viewMode.set("favorites");
        searchBarComponent?.clear();
        goHome();
      });
    } catch (error) {
      console.error("Failed to initialize:", error);
      showToast("Failed to load data", "error");
      isLoading = false;
    }
  });
</script>

<svelte:window on:keydown={handleKeyDown} />

<div class="app">
  <SearchBar bind:this={searchBarComponent} />

  {#if isLoading}
    <div class="loading-container">
      <div class="spinner"></div>
      <p>Loading...</p>
    </div>
  {:else if $viewMode === 'categories'}
    <div class="categories-container">
      {#if $currentCategory}
        <button class="back-button" on:click={() => goHome()}>
          ← Back to Categories
        </button>
      {/if}
      <div class="categories-grid">
        {#each $categories as category, index (category.slug)}
          <CategoryItem
            {category}
            {index}
            selected={$selectedIndex === index}
            onClick={handleCategoryClick}
            onHover={() => selectedIndex.set(index)}
            onLeave={() => selectedIndex.set(-1)}
          />
        {/each}
      </div>
    </div>
  {:else}
    {#if $viewMode === 'category' && $currentCategory}
      <div class="category-header">
        <button class="back-button" on:click={() => { viewMode.set('categories'); loadCategories(); }}>
          ← Back
        </button>
        <span class="category-title">{$currentCategory.name}</span>
      </div>
    {/if}

    {#if $viewMode === 'search' && $searchSuggestions.length > 0}
      <SearchSuggestionsTicker onSelect={(s) => searchBarComponent?.setQuery(s)} />
    {/if}

    <MasonryLayout
      items={allItems}
      onItemClick={handleItemClick}
      onScrollNearEnd={$viewMode !== 'favorites' ? handleScrollNearEnd : undefined}
    />

    {#if $isLoadingMore}
      <div class="loading-more">
        <div class="spinner-small"></div>
        <p>Loading more...</p>
      </div>
    {/if}
  {/if}

  {#if ($viewMode === 'search' || $viewMode === 'trending' || $viewMode === 'category' || $viewMode === 'categories') && (allItems.length > 0 || $categories.length > 0)}
    <a
      href="https://klipy.com"
      target="_blank"
      rel="noopener noreferrer"
      class="klipy-attribution"
    >
      Powered by Klipy
    </a>
  {/if}

  <Toast />
  <ContextMenu />

  {#if $showSettings}
    <Settings />
  {/if}
</div>

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(:root) {
    /* Discord-inspired dark theme */
    --bg-primary: #1e1e1e;
    --bg-secondary: #2d2d2d;
    --bg-tertiary: #3d3d3d;
    --text-primary: #dcddde;
    --text-secondary: #b9bbbe;
    --text-tertiary: #72767d;
    --border-color: #3d3d3d;
    --accent-color: #5865f2;
    --accent-color-light: rgba(88, 101, 242, 0.2);
    --success-color: #3ba55c;
    --error-color: #ed4245;

    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen,
      Ubuntu, Cantarell, sans-serif;
    font-size: 14px;
    line-height: 1.5;
    font-weight: 400;
    color: var(--text-primary);
    background-color: var(--bg-primary);

    -webkit-user-select: none;
    user-select: none;
  }

  @media (prefers-color-scheme: light) {
    :global(:root) {
      --bg-primary: #ffffff;
      --bg-secondary: #f2f3f5;
      --bg-tertiary: #e3e5e8;
      --text-primary: #2e3338;
      --text-secondary: #4f5660;
      --text-tertiary: #747f8d;
      --border-color: #e3e5e8;
      --accent-color: #5865f2;
      --accent-color-light: rgba(88, 101, 242, 0.15);
    }
  }

  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  .loading-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--bg-tertiary);
    border-top-color: var(--accent-color);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin-bottom: 12px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .loading-container p {
    font-size: 14px;
    font-weight: 500;
  }

  .loading-more {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 24px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: 24px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 500;
    z-index: 100;
  }

  .spinner-small {
    width: 16px;
    height: 16px;
    border: 2px solid var(--bg-tertiary);
    border-top-color: var(--accent-color);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .klipy-attribution {
    position: fixed;
    bottom: 12px;
    right: 12px;
    z-index: 50;
    padding: 8px 12px;
    background: rgba(0, 0, 0, 0.85);
    backdrop-filter: blur(8px);
    border-radius: 6px;
    transition: all 0.2s ease;
    cursor: pointer;
    text-decoration: none;
    display: block;
    color: white;
    font-size: 12px;
    font-weight: 500;
  }

  .klipy-attribution:hover {
    background: rgba(0, 0, 0, 0.95);
    transform: scale(1.05);
  }

  .categories-container {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .categories-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    grid-auto-rows: 10px;
    gap: 8px;
  }

  @media (max-width: 900px) {
    .categories-grid {
      grid-template-columns: repeat(3, 1fr);
    }
  }

  @media (max-width: 600px) {
    .categories-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  .category-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }

  .category-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .back-button {
    padding: 6px 12px;
    background: var(--bg-tertiary);
    border: none;
    border-radius: 4px;
    color: var(--text-secondary);
    font-size: 13px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .back-button:hover {
    background: var(--accent-color);
    color: white;
  }
</style>
