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

// Error state
export const searchError = writable<string | null>(null);

// Debounce timer
let searchDebounceTimer: ReturnType<typeof setTimeout>;

// Main search function - always searches both local favorites and Giphy
export async function performSearch(query: string, giphyApiKey?: string) {
  // If empty query, load all favorites
  if (!query.trim()) {
    try {
      const allFavorites = await invoke<Favorite[]>('get_all_favorites');
      searchResults.set({ local: allFavorites, giphy: undefined });
    } catch (error) {
      console.error('Failed to load favorites:', error);
      searchResults.set({ local: [], giphy: undefined });
    }
    return;
  }

  isSearching.set(true);
  searchError.set(null);

  try {
    // Use combined search - searches both local and Giphy
    const result = await invoke<SearchResult>('search_combined', {
      query,
      giphyLimit: 25,
      giphyOffset: 0,
      apiKey: giphyApiKey || undefined
    });
    searchResults.set(result);
  } catch (error) {
    console.error('Search failed:', error);
    searchError.set(error as string);
  } finally {
    isSearching.set(false);
  }
}

// Debounced search
export function debouncedSearch(query: string, giphyApiKey?: string, delay: number = 300) {
  clearTimeout(searchDebounceTimer);
  searchDebounceTimer = setTimeout(() => {
    performSearch(query, giphyApiKey);
  }, delay);
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
  searchQuery.set('');
  searchResults.set({ local: [], giphy: undefined });
  searchError.set(null);
}
