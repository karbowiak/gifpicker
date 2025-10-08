import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Favorite } from '$lib/types';

// Favorites store
function createFavoritesStore() {
  const { subscribe, set, update } = writable<Favorite[]>([]);

  return {
    subscribe,

    // Load all favorites from backend
    async load() {
      try {
        const favorites = await invoke<Favorite[]>('get_all_favorites');
        set(favorites);
        return favorites;
      } catch (error) {
        console.error('Failed to load favorites:', error);
        throw error;
      }
    },

    // Add a new favorite
    async add(favorite: Favorite) {
      try {
        const id = await invoke<number>('add_favorite', { favorite });
        update(favs => [...favs, { ...favorite, id }]);
        return id;
      } catch (error) {
        console.error('Failed to add favorite:', error);
        throw error;
      }
    },

    // Update an existing favorite
    async updateItem(favorite: Favorite) {
      try {
        await invoke('update_favorite', { favorite });
        update(favs => favs.map(f => f.id === favorite.id ? favorite : f));
      } catch (error) {
        console.error('Failed to update favorite:', error);
        throw error;
      }
    },

    // Delete a favorite
    async delete(id: number) {
      try {
        await invoke('delete_favorite', { id });
        update(favs => favs.filter(f => f.id !== id));
      } catch (error) {
        console.error('Failed to delete favorite:', error);
        throw error;
      }
    },

    // Increment use count
    async incrementUseCount(id: number) {
      try {
        await invoke('increment_use_count', { id });
        update(favs => favs.map(f => {
          if (f.id === id) {
            return { ...f, use_count: f.use_count + 1, last_used: new Date().toISOString() };
          }
          return f;
        }));
      } catch (error) {
        console.error('Failed to increment use count:', error);
        throw error;
      }
    },

    // Import a local file
    async importFile(filePath: string) {
      try {
        const favorite = await invoke<Favorite>('import_local_file', { filePath });
        update(favs => [favorite, ...favs]);
        return favorite;
      } catch (error) {
        console.error('Failed to import file:', error);
        throw error;
      }
    },

    // Clear all favorites (for UI reset)
    clear() {
      set([]);
    }
  };
}

export const favorites = createFavoritesStore();
