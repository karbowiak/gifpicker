// TypeScript types matching Rust backend structures

export type MediaType = 'gif' | 'image' | 'video';
export type Source = 'klipy' | 'local' | 'upload';
export type Theme = 'light' | 'dark' | 'system';
export type ClipboardMode = 'file' | 'url';
export type ClipboardFormat = 'gif' | 'mp4';
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
  clipboard_format: ClipboardFormat;
  show_ads: boolean;
}

export interface KlipyGifResult {
  kind: 'gif';
  id: string;
  slug: string;
  title: string;
  url: string;
  gif_url: string;
  mp4_url?: string;
  width: number;
  height: number;
}

// Inline ad item from Klipy. `content` is a self-contained HTML document
// that must render in a sandboxed iframe — it handles its own click/impression
// tracking, so we don't fire any pings ourselves.
export interface KlipyAdResult {
  kind: 'ad';
  width: number;
  height: number;
  content: string;
}

export type KlipyResultItem = KlipyGifResult | KlipyAdResult;

export interface KlipySearchResults {
  items: KlipyResultItem[];
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

// Type guards. The grid mixes favorites, klipy gifs, and ads, so callers
// need to dispatch on shape.
export function isFavorite(item: Favorite | KlipyResultItem): item is Favorite {
  return 'use_count' in item;
}

export function isKlipyAd(item: Favorite | KlipyResultItem): item is KlipyAdResult {
  return 'kind' in item && item.kind === 'ad';
}

export function isKlipyGif(item: Favorite | KlipyResultItem): item is KlipyGifResult {
  return 'kind' in item && item.kind === 'gif';
}

export function isKlipyCategory(item: Favorite | KlipyResultItem | KlipyCategory): item is KlipyCategory {
  return 'slug' in item && 'name' in item && !('title' in item);
}
