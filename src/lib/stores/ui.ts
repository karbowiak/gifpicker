import { writable } from 'svelte/store';
import type { Favorite, KlipyGifResult } from '$lib/types';

// Selected item index in the grid - with logging
// Selected index for keyboard navigation
function createSelectedIndex() {
  const { subscribe, set, update } = writable<number>(-1);

  return {
    subscribe,
    set: (value: number) => {
      set(value);
    },
    update,
    reset: () => set(-1)
  };
}

export const selectedIndex = createSelectedIndex();

// Is search bar focused
export const isSearchFocused = writable<boolean>(true);

// Show settings modal
export const showSettings = writable<boolean>(false);

// Context menu state
export const contextMenu = writable<{
  show: boolean;
  x: number;
  y: number;
  item?: Favorite | KlipyGifResult;
}>({
  show: false,
  x: 0,
  y: 0,
  item: undefined
});

// Show context menu
export function openContextMenu(x: number, y: number, item: Favorite | KlipyGifResult) {
  contextMenu.set({ show: true, x, y, item });
}

// Hide context menu
export function closeContextMenu() {
  contextMenu.update(m => ({ ...m, show: false }));
}

// Toast notifications
export const toast = writable<{
  show: boolean;
  message: string;
  type: 'success' | 'error' | 'info';
}>({
  show: false,
  message: '',
  type: 'info'
});

// Show toast notification
export function showToast(message: string, type: 'success' | 'error' | 'info' = 'info', duration: number = 3000) {
  toast.set({ show: true, message, type });

  setTimeout(() => {
    toast.update(t => ({ ...t, show: false }));
  }, duration);
}

// Loading overlay
export const isLoading = writable<boolean>(false);
export const loadingMessage = writable<string>('');

// Show loading overlay
export function showLoading(message: string = 'Loading...') {
  loadingMessage.set(message);
  isLoading.set(true);
}

// Hide loading overlay
export function hideLoading() {
  isLoading.set(false);
  loadingMessage.set('');
}
