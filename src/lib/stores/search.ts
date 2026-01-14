import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { settings } from '$lib/stores/settings';
import type { Favorite, KlipyGifResult, KlipySearchResults, KlipyCategory, KlipyCategoriesResult, SearchResult, ViewMode } from '$lib/types';

// Current view mode
export const viewMode = writable<ViewMode>('favorites');

// Current category (when viewing a specific category)
export const currentCategory = writable<KlipyCategory | null>(null);

// Search query store
export const searchQuery = writable<string>('');

// Search results
export const searchResults = writable<SearchResult>({
  local: [],
  klipy: undefined
});

// Categories
export const categories = writable<KlipyCategory[]>([]);

// Autocomplete suggestions
export const autocompleteSuggestions = writable<string[]>([]);

// Search suggestions (related searches)
export const searchSuggestions = writable<string[]>([]);

// Loading state
export const isSearching = writable<boolean>(false);

// Loading more state (for infinite scroll)
export const isLoadingMore = writable<boolean>(false);

// Error state
export const searchError = writable<string | null>(null);

// Current page for pagination
let currentPage = 1;
let currentQuery = '';
let totalCount = 0;
let hasMore = true;

// Debounce timer
let searchDebounceTimer: ReturnType<typeof setTimeout>;

// Main search function
export async function performSearch(query: string) {
  // If empty query, load all favorites
  if (!query.trim()) {
    try {
      const allFavorites = await invoke<Favorite[]>('get_all_favorites');
      searchResults.set({ local: allFavorites, klipy: undefined });
      currentQuery = '';
      currentPage = 1;
      currentPage = 1;
      totalCount = 0;
      hasMore = false;
    } catch (error) {
      console.error('Failed to load favorites:', error);
      searchResults.set({ local: [], klipy: undefined });
    }
    return;
  }

  isSearching.set(true);
  searchError.set(null);
  currentQuery = query;
  currentPage = 1;
  currentPage = 1;
  totalCount = 0;
  hasMore = true;

  // Set a timeout to force reset loading state if it gets stuck (10 seconds)
  const timeoutId = setTimeout(() => {
    isSearching.set(false);
  }, 10000);

  try {
    // Get show_ads setting
    const currentSettings = settings.get();
    const showAds = currentSettings?.show_ads ?? true;

    // Search Klipy
    const result = await invoke<KlipySearchResults>('search_klipy', {
      query,
      limit: 50,
      page: 1,
      showAds
    });

    // Set results with empty local array
    searchResults.set({
      local: [],
      klipy: result
    });

    totalCount = result.total_count;
    currentPage = result.page;
  } catch (error) {
    console.error('Search failed:', error);
    searchError.set(error as string);
  } finally {
    clearTimeout(timeoutId);
    isSearching.set(false);
  }
}

// Load more results for infinite scroll
export async function loadMoreResults() {
  // Only load more if we have a query, not already loading, and have more results
  if (!currentQuery || currentPage === 0 || !hasMore) return;

  // Check if we've reached the end of results
  const currentResults = await new Promise<SearchResult>(resolve => {
    searchResults.subscribe(value => resolve(value))();
  });

  const currentGifCount = currentResults.klipy?.gifs.length || 0;


  // If totalCount is valid and we've reached it, stop
  if (totalCount > 0 && currentGifCount >= totalCount) {
    hasMore = false;
    return;
  }

  isLoadingMore.set(true);

  // Set a timeout to force reset loading state if it gets stuck (10 seconds)
  const timeoutId = setTimeout(() => {
    isLoadingMore.set(false);
  }, 10000);

  try {
    // Get show_ads setting
    const currentSettings = settings.get();
    const showAds = currentSettings?.show_ads ?? true;

    // Load next page
    const nextPage = currentPage + 1;
    const result = await invoke<KlipySearchResults>('search_klipy', {
      query: currentQuery,
      limit: 50,
      page: nextPage,
      showAds
    });

    // If no results returned, we've reached the end
    if (!result.gifs || result.gifs.length === 0) {
      hasMore = false;
      return;
    }

    // Append new results to existing ones, filtering out duplicates
    searchResults.update(current => {
      const existingGifs = current.klipy?.gifs || [];
      const newGifs = result.gifs || [];

      // Create a Set of existing slugs for fast lookup
      const existingSlugs = new Set(existingGifs.map(gif => gif.slug));

      // Filter out duplicates from new results
      const uniqueNewGifs = newGifs.filter(gif => !existingSlugs.has(gif.slug));



      return {
        ...current,
        klipy: {
          ...result,
          gifs: [...existingGifs, ...uniqueNewGifs]
        }
      };
    });

    // Update page
    currentPage = nextPage;
  } catch (error) {
    console.error('Failed to load more results:', error);
  } finally {
    clearTimeout(timeoutId);
    isLoadingMore.set(false);
  }
}

// Debounced search
export function debouncedSearch(query: string, delay: number = 300) {
  clearTimeout(searchDebounceTimer);
  searchDebounceTimer = setTimeout(() => {
    performSearch(query);
  }, delay);
}

// Cancel any pending debounced search
export function cancelPendingSearch() {
  clearTimeout(searchDebounceTimer);
}

// Download Klipy GIF
export async function downloadKlipyGif(gif: KlipyGifResult): Promise<Favorite> {
  try {
    const favorite = await invoke<Favorite>('download_klipy_gif', {
      klipySlug: gif.slug,
      gifUrl: gif.gif_url,
      mp4Url: gif.mp4_url,
      title: gif.title,
      width: gif.width,
      height: gif.height
    });
    return favorite;
  } catch (error) {
    console.error('Failed to download Klipy GIF:', error);
    throw error;
  }
}

// Get trending GIFs
export async function getTrending(limit: number = 50, page: number = 1) {
  try {
    // Get show_ads setting
    const currentSettings = settings.get();
    const showAds = currentSettings?.show_ads ?? true;

    const result = await invoke<KlipySearchResults>('get_klipy_trending', {
      limit,
      page,
      showAds
    });
    return result;
  } catch (error) {
    console.error('Failed to get trending GIFs:', error);
    throw error;
  }
}

// Load trending GIFs into results
export async function loadTrending() {
  isSearching.set(true);
  viewMode.set('trending');
  currentCategory.set(null);
  
  try {
    const result = await getTrending(50, 1);
    searchResults.set({
      local: [],
      klipy: result
    });
    currentPage = 1;
    totalCount = result.total_count;
    hasMore = result.gifs.length < result.total_count;
  } catch (error) {
    console.error('Failed to load trending:', error);
  } finally {
    isSearching.set(false);
  }
}

// Get categories
export async function getCategories() {
  try {
    const currentSettings = settings.get();
    const showAds = currentSettings?.show_ads ?? true;

    const result = await invoke<KlipyCategoriesResult>('get_klipy_categories', {
      showAds
    });
    return result.categories;
  } catch (error) {
    console.error('Failed to get categories:', error);
    throw error;
  }
}

// Load categories
export async function loadCategories() {
  isSearching.set(true);
  viewMode.set('categories');
  currentCategory.set(null);
  
  try {
    const cats = await getCategories();
    categories.set(cats);
    searchResults.set({ local: [], klipy: undefined });
  } catch (error) {
    console.error('Failed to load categories:', error);
  } finally {
    isSearching.set(false);
  }
}

// Load GIFs for a specific category (searches by category name)
export async function loadCategoryGifs(category: KlipyCategory) {
  isSearching.set(true);
  viewMode.set('category');
  currentCategory.set(category);
  currentQuery = category.name;
  currentPage = 1;
  
  try {
    const currentSettings = settings.get();
    const showAds = currentSettings?.show_ads ?? true;

    const result = await invoke<KlipySearchResults>('search_klipy', {
      query: category.name,
      limit: 50,
      page: 1,
      showAds
    });

    searchResults.set({
      local: [],
      klipy: result
    });
    totalCount = result.total_count;
    hasMore = result.gifs.length < result.total_count;
  } catch (error) {
    console.error('Failed to load category GIFs:', error);
  } finally {
    isSearching.set(false);
  }
}

// Go back to favorites view
export async function goHome() {
  viewMode.set('favorites');
  currentCategory.set(null);
  clearSearch();
  
  try {
    const allFavorites = await invoke<Favorite[]>('get_all_favorites');
    searchResults.set({ local: allFavorites, klipy: undefined });
  } catch (error) {
    console.error('Failed to load favorites:', error);
  }
}

// Clear search
export function clearSearch() {
  // Cancel any pending debounced searches
  cancelPendingSearch();

  // Reset all state
  searchQuery.set('');
  searchResults.set({ local: [], klipy: undefined });
  searchError.set(null);
  isSearching.set(false);
  isLoadingMore.set(false);
  autocompleteSuggestions.set([]);
  searchSuggestions.set([]);
  currentQuery = '';
  currentPage = 1;
  currentPage = 1;
  totalCount = 0;
  hasMore = false;
}

// Autocomplete debounce timer
let autocompleteTimer: ReturnType<typeof setTimeout>;

// Get autocomplete suggestions
export async function getAutocomplete(query: string) {
  if (!query.trim() || query.length < 2) {
    autocompleteSuggestions.set([]);
    return;
  }

  // Debounce autocomplete
  clearTimeout(autocompleteTimer);
  autocompleteTimer = setTimeout(async () => {
    try {
      const currentSettings = settings.get();
      const showAds = currentSettings?.show_ads ?? true;

      const results = await invoke<string[]>('get_autocomplete', {
        query,
        limit: 8,
        showAds
      });
      autocompleteSuggestions.set(results);
    } catch (error) {
      console.error('Failed to get autocomplete:', error);
      autocompleteSuggestions.set([]);
    }
  }, 150);
}

// Clear autocomplete
export function clearAutocomplete() {
  clearTimeout(autocompleteTimer);
  autocompleteSuggestions.set([]);
}

// Get search suggestions (related searches)
export async function fetchSearchSuggestions(query: string) {
  if (!query.trim()) {
    searchSuggestions.set([]);
    return;
  }

  try {
    const currentSettings = settings.get();
    const showAds = currentSettings?.show_ads ?? true;

    const results = await invoke<string[]>('get_search_suggestions', {
      query,
      limit: 15,
      showAds
    });
    searchSuggestions.set(results);
  } catch (error) {
    console.error('Failed to get search suggestions:', error);
    searchSuggestions.set([]);
  }
}
