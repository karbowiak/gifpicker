import { invoke } from '@tauri-apps/api/core';
import type {
  ClipboardFormat,
  ClipboardMode,
  Favorite,
  KlipyGifResult,
} from '$lib/types';
import { isFavorite } from '$lib/types';

// What we ended up copying — useful for the toast caller.
export type CopyOutcome =
  | { ok: true; via: 'file:gif' | 'file:mp4' | 'url' }
  | { ok: false; reason: string };

/// Resolve which local file to copy for a favorite, given the user's preferred
/// format. If `preferred = mp4` but no mp4 was downloaded, falls back to gif.
function favoriteFilePath(
  fav: Favorite,
  preferred: ClipboardFormat,
): { path: string; via: 'file:gif' | 'file:mp4' } | null {
  if (preferred === 'mp4' && fav.mp4_filepath) {
    return { path: fav.mp4_filepath, via: 'file:mp4' };
  }
  if (fav.filepath) return { path: fav.filepath, via: 'file:gif' };
  // mp4-only favorite (rare; mp4 was preferred but unavailable, no gif either)
  if (fav.mp4_filepath) return { path: fav.mp4_filepath, via: 'file:mp4' };
  return null;
}

/// For Klipy results we haven't favorited yet, pick a URL + filename to
/// download temp-fashion before copying.
function klipyDownloadTarget(
  klipy: KlipyGifResult,
  preferred: ClipboardFormat,
): { url: string; filename: string; via: 'file:gif' | 'file:mp4' } {
  if (preferred === 'mp4' && klipy.mp4_url) {
    return {
      url: klipy.mp4_url,
      filename: `${klipy.slug}.mp4`,
      via: 'file:mp4',
    };
  }
  return {
    url: klipy.gif_url,
    filename: `${klipy.slug}.gif`,
    via: 'file:gif',
  };
}

/// Copy a grid item to the clipboard, honoring the user's `clipboard_mode`
/// (file vs. url) and `clipboard_format` (gif vs. mp4, with gif fallback).
///
/// Returns what was actually copied so the caller can pick an appropriate
/// toast — or report a meaningful failure.
export async function copyItem(
  item: Favorite | KlipyGifResult,
  mode: ClipboardMode,
  format: ClipboardFormat,
): Promise<CopyOutcome> {
  // URL mode: same regardless of preferred format — we just copy a URL.
  if (mode === 'url') {
    const url = isFavorite(item) ? item.gif_url : item.gif_url;
    if (!url) return { ok: false, reason: 'No URL available for this item' };
    await invoke('copy_text_to_clipboard', { text: url });
    return { ok: true, via: 'url' };
  }

  // File mode.
  if (isFavorite(item)) {
    const target = favoriteFilePath(item, format);
    if (!target) {
      // No file on disk — fall back to URL if we have one
      if (item.gif_url) {
        await invoke('copy_text_to_clipboard', { text: item.gif_url });
        return { ok: true, via: 'url' };
      }
      return { ok: false, reason: 'No file or URL available' };
    }
    await invoke('copy_file_path_to_clipboard', { filePath: target.path });
    return { ok: true, via: target.via };
  }

  // Klipy result: download temp file in chosen format, then copy.
  const target = klipyDownloadTarget(item, format);
  try {
    const filePath = await invoke<string>('download_gif_temp', {
      gifUrl: target.url,
      filename: target.filename,
    });
    await invoke('copy_file_path_to_clipboard', { filePath });
    return { ok: true, via: target.via };
  } catch (error) {
    console.error('Failed to download for clipboard:', error);
    // Last-ditch fallback: copy the URL so the user gets *something*.
    await invoke('copy_text_to_clipboard', { text: target.url });
    return { ok: true, via: 'url' };
  }
}
