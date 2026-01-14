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
  } from "$lib/stores/search";
  import { favorites } from "$lib/stores/favorites";
  import { settings } from "$lib/stores/settings";
  import { selectedIndex, showToast, showSettings } from "$lib/stores/ui";
  import type { Favorite, KlipyGifResult, KlipyCategory } from "$lib/types";
  import { isFavorite } from "$lib/types";

  let allItems: (Favorite | KlipyGifResult)[] = [];
  let isLoading = true;
  let searchBarComponent: SearchBar;

  // Combine search results based on view mode
  $: {
    const items: (Favorite | KlipyGifResult)[] = [];
    const previousLength = allItems.length;

    if ($viewMode === 'favorites') {
      // Show favorites
      if ($searchResults.local && $searchResults.local.length > 0) {
        items.push(...$searchResults.local);
      }
    } else if ($viewMode === 'trending' || $viewMode === 'search' || $viewMode === 'category') {
      // Show Klipy results (trending, search, or category)
      if (
        $searchResults.klipy &&
        $searchResults.klipy.gifs &&
        $searchResults.klipy.gifs.length > 0
      ) {
        items.push(...$searchResults.klipy.gifs);
      }
    }
    // For 'categories' view, we show categories grid instead

    allItems = items;

    // Only reset selection if it's out of bounds AND we're not just adding more items (infinite scroll)
    const isAddingMore = items.length > previousLength && previousLength > 0;

    if (!isAddingMore && $selectedIndex >= allItems.length) {
      selectedIndex.set(-1);
    }
  }

  // Fetch search suggestions when in search mode with results
  $: if ($viewMode === 'search' && $searchQuery.trim()) {
    fetchSearchSuggestions($searchQuery);
  }

  // Handle infinite scroll - load more results
  async function handleScrollNearEnd() {
    // Only load more for search/trending/category views
    if ($viewMode === 'favorites' || $viewMode === 'categories' || $isLoadingMore) return;

    await loadMoreResults();
  }

  // Handle item click - copy GIF to clipboard
  async function handleItemClick(item: Favorite | KlipyGifResult) {
    const clipboardMode = $settings?.clipboard_mode || "file";

    try {
      if (isFavorite(item)) {
        const favorite = item as Favorite;

        if (clipboardMode === "file") {
          // Copy the file itself
          if (favorite.filepath) {
            await invoke("copy_file_path_to_clipboard", {
              filePath: favorite.filepath,
            });
          } else if (favorite.gif_url) {
            await invoke("copy_text_to_clipboard", {
              text: favorite.gif_url,
            });
          }
        } else {
          // Copy URL mode
          if (favorite.gif_url) {
            await invoke("copy_text_to_clipboard", {
              text: favorite.gif_url,
            });
          } else {
            showToast("No URL available for this GIF", "error");
            return;
          }
        }

        // Increment use count
        await invoke("increment_use_count", {
          id: favorite.id,
        });

        showToast("Copied to clipboard!", "success");
      } else {
        // For Klipy search results (not favorited yet)
        const klipyResult = item as KlipyGifResult;

        if (clipboardMode === "file") {
          // Download and copy the file (temporary, not saved to favorites)
          try {
            const filePath = await invoke<string>("download_gif_temp", {
              gifUrl: klipyResult.gif_url,
              filename: `${klipyResult.slug}.gif`,
            });
            await invoke("copy_file_path_to_clipboard", {
              filePath: filePath,
            });
            showToast("GIF copied to clipboard!", "success");
          } catch (error) {
            console.error("Failed to download GIF:", error);
            // Fallback to URL copy
            await invoke("copy_text_to_clipboard", {
              text: klipyResult.gif_url,
            });
            showToast("GIF URL copied to clipboard!", "success");
          }
        } else {
          // URL mode - just copy the URL
          await invoke("copy_text_to_clipboard", {
            text: klipyResult.gif_url,
          });
          showToast("GIF URL copied to clipboard!", "success");
        }
      }

      // Close window if setting is enabled
      const closeAfterCopy = $settings?.close_after_selection ?? true;
      if (closeAfterCopy) {
        // Reset search before closing so favorites show on next open
        clearSearch();
        await performSearch(""); // Reload favorites
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

  // Toggle favorite on selected item
  async function toggleFavorite() {
    const current = $selectedIndex;
    if (current < 0 || current >= allItems.length) return;

    const item = allItems[current];

    if (isFavorite(item)) {
      // Remove from favorites
      const favorite = item as Favorite;
      if (favorite.id) {
        try {
          await favorites.delete(favorite.id);
          searchResults.update((r) => ({
            ...r,
            local: r.local.filter((f) => f.id !== favorite.id),
          }));
          showToast("Removed from favorites", "success");
        } catch (e) {
          showToast("Failed to remove", "error");
        }
      }
    } else {
      // Add to favorites
      const klipyResult = item as KlipyGifResult;
      try {
        await invoke("add_klipy_favorite", {
          gifUrl: klipyResult.gif_url,
          mp4Url: klipyResult.mp4_url || null,
          sourceId: klipyResult.slug,
          sourceUrl: klipyResult.url,
          title: klipyResult.title || "Untitled",
          width: klipyResult.width,
          height: klipyResult.height,
        });
        showToast("Added to favorites!", "success");
      } catch (e) {
        showToast("Failed to add", "error");
      }
    }
  }

  // Handle category click - load GIFs for that category
  function handleCategoryClick(category: KlipyCategory) {
    loadCategoryGifs(category);
  }

  // Get number of grid columns based on window width
  function getColumnCount(): number {
    const width = window.innerWidth;
    if (width <= 400) return 1;
    if (width <= 600) return 2;
    if (width <= 900) return 3;
    return 4;
  }

  // Keyboard navigation - grid-based since CSS Grid maintains DOM order = visual order
  function handleKeyDown(event: KeyboardEvent) {
    // Check if user is typing in search
    if (event.target instanceof HTMLInputElement) {
      // If it's an arrow key (Up/Down), we want to blur input and allow navigation
      if (event.key === "ArrowDown" || event.key === "ArrowUp") {
        event.preventDefault();
        (event.target as HTMLInputElement).blur();
        selectedIndex.set(0);

        const container = document.querySelector(".masonry-layout");
        if (container) container.scrollTo({ top: 0, behavior: "smooth" });

        return;
      } else {
        return;
      }
    }

    const itemCount = allItems.length;
    const current = $selectedIndex;
    const cols = getColumnCount();

    switch (event.key) {
      case "ArrowDown":
        event.preventDefault();
        // Move down by column count, or to last item
        if (current + cols < itemCount) {
          selectedIndex.set(current + cols);
        } else if (current < itemCount - 1) {
          selectedIndex.set(itemCount - 1);
        }
        break;

      case "ArrowUp":
        event.preventDefault();
        // Move up by column count, or to first item
        if (current - cols >= 0) {
          selectedIndex.set(current - cols);
        } else if (current > 0) {
          selectedIndex.set(0);
        }
        break;

      case "ArrowRight":
        event.preventDefault();
        if (current + 1 < itemCount) {
          selectedIndex.set(current + 1);
        }
        break;

      case "ArrowLeft":
        event.preventDefault();
        if (current - 1 >= 0) {
          selectedIndex.set(current - 1);
        }
        break;

      case "Enter":
        event.preventDefault();
        if (current >= 0 && current < itemCount) {
          handleItemClick(allItems[current]);
        }
        break;

      case "f":
      case "F":
        event.preventDefault();
        toggleFavorite();
        break;

      case "Escape":
        event.preventDefault();
        clearSearch();
        performSearch(""); // Reset to favorites
        isSearching.set(false);
        isLoadingMore.set(false);
        invoke("close_window").catch(console.error);
        break;
    }
  }

  onMount(async () => {
    try {
      // Load settings
      await settings.load();

      // Load favorites
      await favorites.load();

      // Load all favorites as initial display
      const allFavorites = await invoke<Favorite[]>("get_all_favorites");
      searchResults.set({ local: allFavorites, klipy: undefined });

      isLoading = false;

      // Listen for tray menu "open-settings" event
      await listen("open-settings", () => {
        showSettings.set(true);
      });

      // Listen for "focus-search" event to focus the search bar
      await listen("focus-search", () => {
        if (searchBarComponent) {
          searchBarComponent.focus();
        }
      });

      // Listen for "clear-search" event to clear the search bar
      await listen("clear-search", () => {
        // Force reset all loading states
        isSearching.set(false);
        isLoadingMore.set(false);

        if (searchBarComponent) {
          searchBarComponent.clear();
        }
      });

      // Listen for window focus events to reset state when window is shown
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const currentWindow = getCurrentWindow();

      await currentWindow.onFocusChanged(({ payload: focused }) => {
        if (focused) {
          // Force reset ALL loading states to prevent stuck UI
          isSearching.set(false);
          isLoadingMore.set(false);

          // Reset to home/favorites view
          viewMode.set('favorites');
          
          // Clear search and reset to favorites
          if (searchBarComponent) {
            searchBarComponent.clear();
          }
          
          // Reload favorites
          goHome();
        }
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
    <!-- Categories Grid -->
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
    <!-- GIF Grid (favorites, trending, search, category) -->
    {#if $viewMode === 'category' && $currentCategory}
      <div class="category-header">
        <button class="back-button" on:click={() => { viewMode.set('categories'); loadCategories(); }}>
          ← Back
        </button>
        <span class="category-title">{$currentCategory.name}</span>
      </div>
    {/if}

    <!-- Search suggestions ticker -->
    {#if $viewMode === 'search' && $searchSuggestions.length > 0}
      <SearchSuggestionsTicker onSelect={(s) => { if (searchBarComponent) searchBarComponent.setQuery(s); }} />
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

  <!-- Show Klipy attribution when displaying Klipy results -->
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

    /* Disable text selection for better app feel */
    -webkit-user-select: none;
    user-select: none;
  }

  /* Light mode override */
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
