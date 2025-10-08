// TypeScript types matching Rust backend structures

export type MediaType = 'gif' | 'image' | 'video';
export type Source = 'giphy' | 'tenor' | 'local' | 'upload';
export type Theme = 'light' | 'dark' | 'system';

export interface Favorite {
  id?: number;
  filename: string;
  filepath?: string;
  gif_url?: string;
  media_type: MediaType;
  source?: Source;
  source_id?: string;
  source_url?: string;
  tags: string[];
  custom_tags: string[];
  description?: string;
  width?: number;
  height?: number;
  file_size?: number;
  created_at: string;
  last_used?: string;
  use_count: number;
}

export interface Settings {
  giphy_api_key?: string;
  hotkey: string;
  window_width: number;
  window_height: number;
  max_item_width: number;
  close_after_selection: boolean;
  launch_at_startup: boolean;
  theme: Theme;
}

export interface GiphyGifResult {
  id: string;
  title: string;
  url: string;
  gif_url: string;
  width: string;
  height: string;
}

export interface GiphySearchResults {
  gifs: GiphyGifResult[];
  total_count: number;
  offset: number;
}

export interface SearchResult {
  local: Favorite[];
  giphy?: GiphySearchResults;
}

// UI-specific types
export interface UIState {
  selectedIndex: number;
  isSearchFocused: boolean;
  showSettings: boolean;
  showContextMenu: boolean;
  contextMenuPosition: { x: number; y: number };
  contextMenuItem?: Favorite | GiphyGifResult;
}

export interface LoadingState {
  isLoading: boolean;
  message?: string;
}

export interface ErrorState {
  hasError: boolean;
  message?: string;
}

// Type guard to check if item is a Favorite
export function isFavorite(item: Favorite | GiphyGifResult): item is Favorite {
  return 'filepath' in item;
}

// Type guard to check if item is a Giphy result
export function isGiphyResult(item: Favorite | GiphyGifResult): item is GiphyGifResult {
  return 'gif_url' in item;
}
