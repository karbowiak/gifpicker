<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import SearchBar from '$lib/components/SearchBar.svelte';
  import MasonryLayout from '$lib/components/MasonryLayout.svelte';
  import Toast from '$lib/components/Toast.svelte';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import { searchResults, searchQuery } from '$lib/stores/search';
  import { favorites } from '$lib/stores/favorites';
  import { settings } from '$lib/stores/settings';
  import { selectedIndex, showToast, showSettings } from '$lib/stores/ui';
  import type { Favorite, GiphyGifResult } from '$lib/types';

  let allItems: (Favorite | GiphyGifResult)[] = [];
  let isLoading = true;

  // Combine search results
  $: {
    const items: (Favorite | GiphyGifResult)[] = [];

    // Add local results
    if ($searchResults.local && $searchResults.local.length > 0) {
      items.push(...$searchResults.local);
    }

    // Add Giphy results
    if ($searchResults.giphy && $searchResults.giphy.gifs && $searchResults.giphy.gifs.length > 0) {
      items.push(...$searchResults.giphy.gifs);
    }

    allItems = items;

    // Reset selection when items change
    if ($selectedIndex >= allItems.length) {
      selectedIndex.set(0);
    }
  }

  // Handle item click
  async function handleItemClick(item: Favorite | GiphyGifResult) {
    // The MediaItem component already handles clipboard copy
    // Just close the window if setting is enabled
    const closeAfterCopy = $settings?.close_after_selection ?? true;

    if (closeAfterCopy) {
      setTimeout(async () => {
        try {
          await invoke('close_window');
        } catch (error) {
          console.error('Failed to close window:', error);
        }
      }, 500); // Small delay to show success toast
    }
  }

  // Keyboard navigation
  function handleKeyDown(event: KeyboardEvent) {
    const itemCount = allItems.length;
    if (itemCount === 0) return;

    const current = $selectedIndex;

    // Calculate items per row (rough estimate based on masonry layout)
    const itemsPerRow = 4;

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        if (current + itemsPerRow < itemCount) {
          selectedIndex.set(current + itemsPerRow);
        }
        break;

      case 'ArrowUp':
        event.preventDefault();
        if (current - itemsPerRow >= 0) {
          selectedIndex.set(current - itemsPerRow);
        }
        break;

      case 'ArrowRight':
        event.preventDefault();
        if (current + 1 < itemCount) {
          selectedIndex.set(current + 1);
        }
        break;

      case 'ArrowLeft':
        event.preventDefault();
        if (current - 1 >= 0) {
          selectedIndex.set(current - 1);
        }
        break;

      case 'Enter':
        event.preventDefault();
        if (current >= 0 && current < itemCount) {
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
    } catch (error) {
      console.error('Failed to initialize:', error);
      showToast('Failed to load data', 'error');
      isLoading = false;
    }
  });
</script>

<svelte:window on:keydown={handleKeyDown} />

<div class="app">
  <SearchBar />

  {#if isLoading}
    <div class="loading-container">
      <div class="spinner"></div>
      <p>Loading...</p>
    </div>
  {:else}
    <MasonryLayout items={allItems} onItemClick={handleItemClick} />
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
</style>
