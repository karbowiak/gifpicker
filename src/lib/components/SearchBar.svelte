<script lang="ts">
  import { onMount } from 'svelte';
  import {
    searchQuery as searchQueryStore,
    performSearch,
    debouncedSearch,
    clearSearch as clearSearchStore
  } from '$lib/stores/search';
  import { settings } from '$lib/stores/settings';
  import { selectedIndex, showSettings } from '$lib/stores/ui';

  let inputElement: HTMLInputElement;
  let query = '';
  let apiKey: string | undefined = undefined;

  // Subscribe to settings to get API key
  settings.subscribe($settings => {
    apiKey = $settings?.giphy_api_key;
  });

  onMount(() => {
    // Auto-focus the search input when mounted
    inputElement?.focus();
  });

  // Open settings
  function openSettings() {
    showSettings.set(true);
  }

  // Handle search input changes with debouncing
  function handleInput(event: Event) {
    const target = event.target as HTMLInputElement;
    query = target.value;
    searchQueryStore.set(query);

    // Debounce the search
    debouncedSearch(query, apiKey);

    // Reset selection when search changes
    selectedIndex.set(0);
  }

  // Handle Enter key - perform immediate search
  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault();
      performSearch(query, apiKey);
      selectedIndex.set(0);
    }
  }

  // Clear search input
  function clearSearch() {
    query = '';
    clearSearchStore();
    performSearch('', apiKey); // Load all favorites
    selectedIndex.set(0);
    inputElement?.focus();
  }
</script>

<div class="search-bar">
  <div class="search-input-wrapper">
    <svg class="search-icon" width="20" height="20" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M9 17A8 8 0 1 0 9 1a8 8 0 0 0 0 16zM18 18l-4-4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>

    <input
      bind:this={inputElement}
      type="text"
      class="search-input"
      placeholder="Search GIFs..."
      value={query}
      on:input={handleInput}
      on:keydown={handleKeyDown}
    />

    {#if query}
      <button class="clear-button" on:click={clearSearch} aria-label="Clear search">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M12 4L4 12M4 4l8 8" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </button>
    {/if}
  </div>

  <button class="settings-button" on:click={openSettings} aria-label="Settings">
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M12 15a3 3 0 100-6 3 3 0 000 6z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>
  </button>
</div>

<style>
  .search-bar {
    padding: 16px;
    background: var(--bg-primary, #ffffff);
    border-bottom: 1px solid var(--border-color, #e5e7eb);
    position: sticky;
    top: 0;
    z-index: 10;
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .search-input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    flex: 1;
  }

  .search-icon {
    position: absolute;
    left: 12px;
    color: var(--text-secondary, #6b7280);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: 10px 40px 10px 44px;
    font-size: 14px;
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 8px;
    background: var(--bg-secondary, #f9fafb);
    color: var(--text-primary, #111827);
    transition: all 0.2s ease;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--accent-color, #3b82f6);
    background: var(--bg-primary, #ffffff);
    box-shadow: 0 0 0 3px var(--accent-color-light, rgba(59, 130, 246, 0.1));
  }

  .search-input::placeholder {
    color: var(--text-tertiary, #9ca3af);
  }

  .clear-button {
    position: absolute;
    right: 12px;
    padding: 4px;
    background: none;
    border: none;
    color: var(--text-secondary, #6b7280);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    transition: all 0.2s ease;
  }

  .clear-button:hover {
    background: var(--bg-tertiary, #f3f4f6);
    color: var(--text-primary, #111827);
  }

  .settings-button {
    padding: 10px;
    background: none;
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 8px;
    color: var(--text-secondary, #6b7280);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
    flex-shrink: 0;
  }

  .settings-button:hover {
    background: var(--bg-secondary, #f9fafb);
    color: var(--text-primary, #111827);
    border-color: var(--accent-color, #3b82f6);
  }

  .settings-button:active {
    transform: scale(0.95);
  }
</style>
