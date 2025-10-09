import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Favorite, GiphyGifResult, GiphySearchResults, SearchResult } from '$lib/types';

// Search query store
export const searchQuery = writable<string>('');

// Search results
export const searchResults = writable<SearchResult>({
  local: [],
  giphy: undefined
});

// Loading state
export const isSearching = writable<boolean>(false);

// Loading more state (for infinite scroll)
export const isLoadingMore = writable<boolean>(false);

// Error state
export const searchError = writable<string | null>(null);

// Current offset for pagination
let currentOffset = 0;
let currentQuery = '';
let totalCount = 0;

// Giphy API has a maximum offset of 4999
const MAX_GIPHY_OFFSET = 4999;

// Debounce timer
let searchDebounceTimer: ReturnType<typeof setTimeout>;

// Main search function - only searches Giphy when there's a query
export async function performSearch(query: string, giphyApiKey?: string) {
  // If empty query, load all favorites
  if (!query.trim()) {
    try {
      const allFavorites = await invoke<Favorite[]>('get_all_favorites');
      searchResults.set({ local: allFavorites, giphy: undefined });
      currentQuery = '';
      currentOffset = 0;
      totalCount = 0;
    } catch (error) {
      console.error('Failed to load favorites:', error);
      searchResults.set({ local: [], giphy: undefined });
    }
    return;
  }

  isSearching.set(true);
  searchError.set(null);
  currentQuery = query;
  currentOffset = 0;
  totalCount = 0;

  // Set a timeout to force reset loading state if it gets stuck (10 seconds)
  const timeoutId = setTimeout(() => {
    isSearching.set(false);
  }, 10000);

  try {
    // Search only Giphy (no local favorites)
    // Start with 25 results (similar to Giphy's own interface)
    const result = await invoke<GiphySearchResults>('search_giphy', {
      query,
      limit: 25,
      offset: 0,
      apiKey: giphyApiKey || undefined
    });

    // Set results with empty local array
    searchResults.set({
      local: [],
      giphy: result
    });

    totalCount = result.total_count;
    // Set offset to the actual number of results we have
    currentOffset = result.gifs.length;
  } catch (error) {
    console.error('Search failed:', error);
    searchError.set(error as string);
  } finally {
    clearTimeout(timeoutId);
    isSearching.set(false);
  }
}

// Load more results for infinite scroll
export async function loadMoreResults(giphyApiKey?: string) {
  // Only load more if we have a query and we're not already loading
  if (!currentQuery || currentOffset === 0) return;

  // Check if we've reached Giphy's limit or the end of results
  if (currentOffset >= MAX_GIPHY_OFFSET || currentOffset >= totalCount) {
    return;
  }

  isLoadingMore.set(true);

  // Set a timeout to force reset loading state if it gets stuck (10 seconds)
  const timeoutId = setTimeout(() => {
    isLoadingMore.set(false);
  }, 10000);

  try {
    // Load next batch (25 results like Giphy's interface)
    const result = await invoke<GiphySearchResults>('search_giphy', {
      query: currentQuery,
      limit: 25,
      offset: currentOffset,
      apiKey: giphyApiKey || undefined
    });

    // If no results returned, we've reached the end
    if (!result.gifs || result.gifs.length === 0) {
      return;
    }

    // Append new results to existing ones, filtering out duplicates
    searchResults.update(current => {
      const existingGifs = current.giphy?.gifs || [];
      const newGifs = result.gifs || [];
      
      // Create a Set of existing IDs for fast lookup
      const existingIds = new Set(existingGifs.map(gif => gif.id));
      
      // Filter out duplicates from new results
      const uniqueNewGifs = newGifs.filter(gif => !existingIds.has(gif.id));

      return {
        ...current,
        giphy: {
          ...result,
          gifs: [...existingGifs, ...uniqueNewGifs]
        }
      };
    });

    // Update offset to the actual number of GIFs we now have
    currentOffset += result.gifs.length;
  } catch (error) {
    console.error('Failed to load more results:', error);
  } finally {
    clearTimeout(timeoutId);
    isLoadingMore.set(false);
  }
}

// Debounced search
export function debouncedSearch(query: string, giphyApiKey?: string, delay: number = 300) {
  clearTimeout(searchDebounceTimer);
  searchDebounceTimer = setTimeout(() => {
    performSearch(query, giphyApiKey);
  }, delay);
}

// Cancel any pending debounced search
export function cancelPendingSearch() {
  clearTimeout(searchDebounceTimer);
}

// Download Giphy GIF
export async function downloadGiphyGif(gif: GiphyGifResult): Promise<Favorite> {
  try {
    const favorite = await invoke<Favorite>('download_giphy_gif', {
      giphyId: gif.id,
      gifUrl: gif.gif_url,
      title: gif.title,
      width: gif.width,
      height: gif.height
    });
    return favorite;
  } catch (error) {
    console.error('Failed to download Giphy GIF:', error);
    throw error;
  }
}

// Get trending GIFs
export async function getTrending(giphyApiKey: string, limit: number = 20, offset: number = 0) {
  try {
    const result = await invoke<GiphySearchResults>('get_giphy_trending', {
      limit,
      offset,
      apiKey: giphyApiKey
    });
    return result;
  } catch (error) {
    console.error('Failed to get trending GIFs:', error);
    throw error;
  }
}

// Clear search
export function clearSearch() {
  // Cancel any pending debounced searches
  cancelPendingSearch();
  
  // Reset all state
  searchQuery.set('');
  searchResults.set({ local: [], giphy: undefined });
  searchError.set(null);
  isSearching.set(false);
  isLoadingMore.set(false);
  currentQuery = '';
  currentOffset = 0;
  totalCount = 0;
}
