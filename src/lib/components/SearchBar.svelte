<script lang="ts">
  import { onMount } from "svelte";
  import {
    searchQuery as searchQueryStore,
    performSearch,
    debouncedSearch,
    clearSearch as clearSearchStore,
    cancelPendingSearch,
    isLoadingMore,
    isSearching,
    viewMode,
    goHome,
    loadTrending,
    loadCategories,
    autocompleteSuggestions,
    getAutocomplete,
    clearAutocomplete,
  } from "$lib/stores/search";
  import { selectedIndex, showSettings } from "$lib/stores/ui";

  let inputElement: HTMLInputElement;
  let query = "";
  let inlineSuggestion = ""; // The ghost text suggestion

  // Compute the ghost text (part after what user typed)
  $: {
    if ($autocompleteSuggestions.length > 0 && query.trim()) {
      const firstSuggestion = $autocompleteSuggestions[0];
      // Check if suggestion starts with query (case insensitive)
      if (firstSuggestion.toLowerCase().startsWith(query.toLowerCase())) {
        inlineSuggestion = firstSuggestion.slice(query.length);
      } else {
        inlineSuggestion = "";
      }
    } else {
      inlineSuggestion = "";
    }
  }

  onMount(() => {
    inputElement?.focus();
  });

  export function focus() {
    inputElement?.focus();
  }

  export function setQuery(newQuery: string) {
    query = newQuery;
    inlineSuggestion = "";
  }

  export function clear() {
    cancelPendingSearch();
    isSearching.set(false);
    isLoadingMore.set(false);
    query = "";
    inlineSuggestion = "";
    clearSearchStore();
    performSearch("");
    selectedIndex.set(-1);
  }

  function openSettings() {
    showSettings.set(true);
  }

  function handleInput(event: Event) {
    const target = event.target as HTMLInputElement;
    query = target.value;
    searchQueryStore.set(query);
    if (query.trim()) {
      viewMode.set('search');
      getAutocomplete(query);
    } else {
      clearAutocomplete();
      inlineSuggestion = "";
    }
    debouncedSearch(query);
    selectedIndex.set(-1);
  }

  function handleKeyDown(event: KeyboardEvent) {
    // Tab to accept inline suggestion
    if (event.key === "Tab" && inlineSuggestion) {
      event.preventDefault();
      const fullQuery = query + inlineSuggestion;
      query = fullQuery;
      searchQueryStore.set(query);
      inlineSuggestion = "";
      clearAutocomplete();
      viewMode.set('search');
      performSearch(query);
      return;
    }

    // Enter to search with current query
    if (event.key === "Enter") {
      event.preventDefault();
      if (query.trim()) {
        clearAutocomplete();
        inlineSuggestion = "";
        performSearch(query);
        inputElement?.blur();
      }
      return;
    }

    // Escape to clear
    if (event.key === "Escape") {
      clearAutocomplete();
      inlineSuggestion = "";
    }
  }

  function clearSearch() {
    cancelPendingSearch();
    isSearching.set(false);
    isLoadingMore.set(false);
    query = "";
    inlineSuggestion = "";
    clearSearchStore();
    performSearch("");
    selectedIndex.set(-1);
    inputElement?.focus();
  }

  function handleHome() {
    query = "";
    inlineSuggestion = "";
    goHome();
    selectedIndex.set(-1);
  }

  function handleTrending() {
    query = "";
    inlineSuggestion = "";
    loadTrending();
    selectedIndex.set(-1);
  }

  function handleCategories() {
    query = "";
    inlineSuggestion = "";
    loadCategories();
    selectedIndex.set(-1);
  }
</script>

<div class="search-bar">
  <button
    class="nav-button"
    class:active={$viewMode === 'favorites'}
    on:click={handleHome}
    aria-label="Home"
    title="Favorites"
  >
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
      <path
        d="M3 9l9-7 9 7v11a2 2 0 01-2 2H5a2 2 0 01-2-2z"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <polyline
        points="9 22 9 12 15 12 15 22"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </button>

  <div class="search-input-wrapper">
    <svg
      class="search-icon"
      width="16"
      height="16"
      viewBox="0 0 20 20"
      fill="none"
    >
      <path
        d="M9 17A8 8 0 1 0 9 1a8 8 0 0 0 0 16zM18 18l-4-4"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
      />
    </svg>

    <input
      bind:this={inputElement}
      type="text"
      class="search-input"
      placeholder={inlineSuggestion ? "" : "Search GIFs..."}
      value={query}
      on:input={handleInput}
      on:keydown={handleKeyDown}
      autocomplete="off"
      spellcheck="false"
    />

    <!-- Ghost text overlay - positioned on top of input -->
    {#if inlineSuggestion}
      <div class="ghost-text-wrapper">
        <span class="ghost-text-typed">{query}</span><span class="ghost-text-suggestion">{inlineSuggestion}</span>
      </div>
    {/if}

    {#if query}
      <button class="clear-button" on:click={clearSearch} aria-label="Clear">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path
            d="M12 4L4 12M4 4l8 8"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
          />
        </svg>
      </button>
    {/if}
  </div>

  <button
    class="nav-button"
    class:active={$viewMode === 'trending'}
    on:click={handleTrending}
    aria-label="Trending"
    title="Trending"
  >
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
      <polyline
        points="23 6 13.5 15.5 8.5 10.5 1 18"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <polyline
        points="17 6 23 6 23 12"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </button>

  <button
    class="nav-button"
    class:active={$viewMode === 'categories' || $viewMode === 'category'}
    on:click={handleCategories}
    aria-label="Categories"
    title="Categories"
  >
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
      <rect x="3" y="3" width="7" height="7" rx="1" stroke="currentColor" stroke-width="2" />
      <rect x="14" y="3" width="7" height="7" rx="1" stroke="currentColor" stroke-width="2" />
      <rect x="3" y="14" width="7" height="7" rx="1" stroke="currentColor" stroke-width="2" />
      <rect x="14" y="14" width="7" height="7" rx="1" stroke="currentColor" stroke-width="2" />
    </svg>
  </button>

  <button class="nav-button" on:click={openSettings} aria-label="Settings" title="Settings">
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
      <path
        d="M12 15a3 3 0 100-6 3 3 0 000 6z"
        stroke="currentColor"
        stroke-width="2"
      />
      <path
        d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"
        stroke="currentColor"
        stroke-width="2"
      />
    </svg>
  </button>
</div>

<style>
  .search-bar {
    padding: 8px;
    background: var(--bg-primary);
    display: flex;
    gap: 8px;
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
    left: 10px;
    color: var(--text-tertiary);
    pointer-events: none;
    z-index: 2;
  }

  .ghost-text-wrapper {
    position: absolute;
    left: 36px;
    right: 32px;
    top: 50%;
    transform: translateY(-50%);
    pointer-events: none;
    font-size: 13px;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    white-space: nowrap;
    overflow: hidden;
    z-index: 3;
    line-height: 1;
  }

  .ghost-text-typed {
    visibility: hidden;
  }

  .ghost-text-suggestion {
    color: var(--text-tertiary);
  }

  .search-input {
    width: 100%;
    padding: 8px 32px 8px 36px;
    font-size: 13px;
    border: none;
    border-radius: 4px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    transition: background 0.15s ease;
    position: relative;
    z-index: 2;
  }

  .search-input:focus {
    outline: none;
    background: var(--bg-tertiary);
  }

  .search-input::placeholder {
    color: var(--text-tertiary);
  }

  .clear-button {
    position: absolute;
    right: 8px;
    padding: 4px;
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 3px;
    transition: all 0.15s ease;
    z-index: 3;
  }

  .clear-button:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .nav-button {
    padding: 8px;
    background: var(--bg-secondary);
    border: none;
    border-radius: 4px;
    color: var(--text-tertiary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
    flex-shrink: 0;
  }

  .nav-button:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .nav-button.active {
    background: var(--accent-color);
    color: white;
  }
</style>
