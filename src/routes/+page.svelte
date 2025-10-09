<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import SearchBar from '$lib/components/SearchBar.svelte';
  import MasonryLayout from '$lib/components/MasonryLayout.svelte';
  import Toast from '$lib/components/Toast.svelte';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import { searchResults, searchQuery, loadMoreResults, isLoadingMore } from '$lib/stores/search';
  import { favorites } from '$lib/stores/favorites';
  import { settings } from '$lib/stores/settings';
  import { selectedIndex, showToast, showSettings } from '$lib/stores/ui';
  import type { Favorite, GiphyGifResult } from '$lib/types';
  import { isFavorite } from '$lib/types';

  let allItems: (Favorite | GiphyGifResult)[] = [];
  let isLoading = true;
  let searchBarComponent: SearchBar;
  let hasSearchQuery = false;

  // Combine search results
  $: {
    const items: (Favorite | GiphyGifResult)[] = [];
    const previousLength = allItems.length;

    // Track if we have a search query
    hasSearchQuery = !!$searchQuery.trim();

    // When searching, ONLY show Giphy results (no favorites)
    if (hasSearchQuery) {
      if ($searchResults.giphy && $searchResults.giphy.gifs && $searchResults.giphy.gifs.length > 0) {
        items.push(...$searchResults.giphy.gifs);
      }
    } else {
      // No search query - show favorites only
      if ($searchResults.local && $searchResults.local.length > 0) {
        items.push(...$searchResults.local);
      }
    }

    console.log('allItems updating, old length:', previousLength, 'new length:', items.length, 'current selectedIndex:', $selectedIndex);
    allItems = items;

    // Only reset selection if it's out of bounds AND we're not just adding more items (infinite scroll)
    // If we're adding items (new length > old length), keep the current selection
    const isLoadingMore = items.length > previousLength && previousLength > 0;

    if (!isLoadingMore && $selectedIndex >= allItems.length) {
      console.log('Resetting selectedIndex from', $selectedIndex, 'to 0 because items changed');
      selectedIndex.set(0);
    }
  }

  // Handle infinite scroll - load more Giphy results
  async function handleScrollNearEnd() {
    // Only load more if we have a search query and we're not already loading
    if (!hasSearchQuery || $isLoadingMore) return;

    const apiKey = $settings?.giphy_api_key;
    if (!apiKey) return;

    await loadMoreResults(apiKey);
  }

  // Handle item click - copy GIF to clipboard
  async function handleItemClick(item: Favorite | GiphyGifResult) {
    console.log('handleItemClick called with:', item);
    const clipboardMode = $settings?.clipboard_mode || 'file';

    try {
      if (isFavorite(item)) {
        const favorite = item as Favorite;

        if (clipboardMode === 'file') {
          // Copy the file itself
          if (favorite.filepath) {
            await invoke('copy_file_path_to_clipboard', {
              filePath: favorite.filepath
            });
          } else if (favorite.gif_url) {
            await invoke('copy_text_to_clipboard', {
              text: favorite.gif_url
            });
          }
        } else {
          // Copy URL mode
          if (favorite.gif_url) {
            await invoke('copy_text_to_clipboard', {
              text: favorite.gif_url
            });
          } else {
            showToast('No Giphy URL available for this GIF', 'error');
            return;
          }
        }

        // Increment use count
        await invoke('increment_use_count', {
          id: favorite.id
        });

        showToast('Copied to clipboard!', 'success');
      } else {
        // For Giphy search results (not favorited yet)
        const giphyResult = item as GiphyGifResult;

        if (clipboardMode === 'file') {
          // Download and copy the file (temporary, not saved to favorites)
          try {
            const filePath = await invoke<string>('download_gif_temp', {
              gifUrl: giphyResult.gif_url,
              filename: `${giphyResult.id}.gif`
            });
            await invoke('copy_file_path_to_clipboard', {
              filePath: filePath
            });
            showToast('GIF copied to clipboard!', 'success');
          } catch (error) {
            console.error('Failed to download GIF:', error);
            // Fallback to URL copy
            await invoke('copy_text_to_clipboard', {
              text: giphyResult.gif_url
            });
            showToast('GIF URL copied to clipboard!', 'success');
          }
        } else {
          // URL mode - just copy the URL
          await invoke('copy_text_to_clipboard', {
            text: giphyResult.gif_url
          });
          showToast('GIF URL copied to clipboard!', 'success');
        }
      }

      // Close window if setting is enabled
      const closeAfterCopy = $settings?.close_after_selection ?? true;
      if (closeAfterCopy) {
        // Close immediately - no delay needed
        try {
          await invoke('close_window');
        } catch (error) {
          console.error('Failed to close window:', error);
        }
      }
    } catch (error) {
      console.error('Failed to copy to clipboard:', error);
      showToast('Failed to copy to clipboard', 'error');
    }
  }

  // Keyboard navigation
  function handleKeyDown(event: KeyboardEvent) {
    const itemCount = allItems.length;
    if (itemCount === 0) return;

    const current = $selectedIndex;
    console.log('KeyDown:', event.key, 'current selectedIndex:', current, 'itemCount:', itemCount);

    // Calculate items per row (rough estimate based on masonry layout)
    const itemsPerRow = 4;

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        if (current + itemsPerRow < itemCount) {
          console.log('ArrowDown: setting to', current + itemsPerRow);
          selectedIndex.set(current + itemsPerRow);
        }
        break;

      case 'ArrowUp':
        event.preventDefault();
        if (current - itemsPerRow >= 0) {
          console.log('ArrowUp: setting to', current - itemsPerRow);
          selectedIndex.set(current - itemsPerRow);
        }
        break;

      case 'ArrowRight':
        event.preventDefault();
        if (current + 1 < itemCount) {
          console.log('ArrowRight: setting to', current + 1);
          selectedIndex.set(current + 1);
        }
        break;

      case 'ArrowLeft':
        event.preventDefault();
        if (current - 1 >= 0) {
          console.log('ArrowLeft: setting to', current - 1);
          selectedIndex.set(current - 1);
        }
        break;

      case 'Enter':
        event.preventDefault();
        if (current >= 0 && current < itemCount) {
          console.log('Enter pressed - selectedIndex:', current, 'itemCount:', itemCount);
          console.log('Selected item:', allItems[current]);
          handleItemClick(allItems[current]);
        }
        break;

      case 'Escape':
        event.preventDefault();
        // Close window
        invoke('close_window').catch(console.error);
        break;
    }
  }

  onMount(async () => {
    try {
      // Load settings first
      const loadedSettings = await settings.load();

      // Check if API key is missing and force settings open
      if (!loadedSettings?.giphy_api_key) {
        showSettings.set(true);
      }

      // Load favorites
      await favorites.load();

      // Load all favorites as initial display
      const allFavorites = await invoke<Favorite[]>('get_all_favorites');
      searchResults.set({ local: allFavorites, giphy: undefined });

      isLoading = false;

      // Listen for tray menu "open-settings" event
      await listen('open-settings', () => {
        showSettings.set(true);
      });

      // Listen for "focus-search" event to focus the search bar
      await listen('focus-search', () => {
        if (searchBarComponent) {
          searchBarComponent.focus();
        }
      });

      // Listen for "clear-search" event to clear the search bar
      await listen('clear-search', () => {
        if (searchBarComponent) {
          searchBarComponent.clear();
        }
      });
    } catch (error) {
      console.error('Failed to initialize:', error);
      showToast('Failed to load data', 'error');
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
  {:else}
    <MasonryLayout
      items={allItems}
      onItemClick={handleItemClick}
      onScrollNearEnd={hasSearchQuery ? handleScrollNearEnd : undefined}
    />

    {#if $isLoadingMore}
      <div class="loading-more">
        <div class="spinner-small"></div>
        <p>Loading more...</p>
      </div>
    {/if}
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
    /* Color variables - Light theme */
    --bg-primary: #ffffff;
    --bg-secondary: #f9fafb;
    --bg-tertiary: #f3f4f6;
    --text-primary: #111827;
    --text-secondary: #6b7280;
    --text-tertiary: #9ca3af;
    --border-color: #e5e7eb;
    --accent-color: #3b82f6;
    --accent-color-light: rgba(59, 130, 246, 0.1);

    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    font-size: 14px;
    line-height: 1.5;
    font-weight: 400;
    color: var(--text-primary);
    background-color: var(--bg-primary);

    /* Disable text selection for better app feel */
    -webkit-user-select: none;
    user-select: none;
  }

  /* Dark mode */
  @media (prefers-color-scheme: dark) {
    :global(:root) {
      --bg-primary: #1f2937;
      --bg-secondary: #111827;
      --bg-tertiary: #374151;
      --text-primary: #f9fafb;
      --text-secondary: #d1d5db;
      --text-tertiary: #9ca3af;
      --border-color: #374151;
      --accent-color: #60a5fa;
      --accent-color-light: rgba(96, 165, 250, 0.15);
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
</style>
