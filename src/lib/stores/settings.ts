import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Settings } from '$lib/types';

// Default settings
const defaultSettings: Settings = {
  hotkey: 'Option+Cmd+G',
  window_width: 800,
  window_height: 600,
  max_item_width: 400,
  close_after_selection: true,
  launch_at_startup: false,
  theme: 'system',
  clipboard_mode: 'file',
  clipboard_format: 'gif',
  show_ads: true,
  tile_size: 'medium',
  always_on_top: false
};

// Settings store
function createSettingsStore() {
  const store = writable<Settings>(defaultSettings);
  const { subscribe, set, update } = store;

  return {
    subscribe,
    get: () => get(store),

    async load() {
      try {
        const settings = await invoke<Settings>('get_settings');
        set(settings);
        return settings;
      } catch (error) {
        console.error('Failed to load settings:', error);
        set(defaultSettings);
        throw error;
      }
    },

    async save(settings: Settings) {
      try {
        await invoke('save_settings', { settings });
        set(settings);
      } catch (error) {
        console.error('Failed to save settings:', error);
        throw error;
      }
    },

    async updateSetting<K extends keyof Settings>(key: K, value: Settings[K]) {
      try {
        await invoke('update_setting', { key, value: JSON.stringify(value) });
        update(s => ({ ...s, [key]: value }));
      } catch (error) {
        console.error('Failed to update setting:', error);
        throw error;
      }
    },

    reset() {
      set(defaultSettings);
    }
  };
}

export const settings = createSettingsStore();
