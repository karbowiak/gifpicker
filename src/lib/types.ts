// TypeScript types matching Rust backend structures

export type MediaType = 'gif' | 'image' | 'video';
export type Source = 'klipy' | 'local' | 'upload';
export type Theme = 'light' | 'dark' | 'system';
export type ClipboardMode = 'file' | 'url';
export type ViewMode = 'favorites' | 'trending' | 'categories' | 'category' | 'search';

export interface Favorite {
  id?: number;
  filename: string;
  filepath?: string;      // GIF file (for clipboard)
  mp4_filepath?: string;  // MP4 for UI display
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
  hotkey: string;
  window_width: number;
  window_height: number;
  max_item_width: number;
  close_after_selection: boolean;
  launch_at_startup: boolean;
  theme: Theme;
  clipboard_mode: ClipboardMode;
  show_ads: boolean;
}

export interface KlipyGifResult {
  id: string;
  slug: string;
  title: string;
  url: string;
  gif_url: string;
  mp4_url?: string;
  width: number;
  height: number;
}

export interface KlipySearchResults {
  gifs: KlipyGifResult[];
  total_count: number;
  page: number;
}

export interface KlipyCategory {
  name: string;
  slug: string;
  gif_url: string;
  mp4_url?: string;
  width: number;
  height: number;
}

export interface KlipyCategoriesResult {
  categories: KlipyCategory[];
}

export interface SearchResult {
  local: Favorite[];
  klipy?: KlipySearchResults;
}

// Type guards
export function isFavorite(item: Favorite | KlipyGifResult): item is Favorite {
  return 'filepath' in item || 'use_count' in item;
}

export function isKlipyResult(item: Favorite | KlipyGifResult): item is KlipyGifResult {
  return 'slug' in item && !('use_count' in item);
}

export function isKlipyCategory(item: Favorite | KlipyGifResult | KlipyCategory): item is KlipyCategory {
  return 'slug' in item && 'name' in item && !('title' in item);
}
