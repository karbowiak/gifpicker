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
  show_ads: true
};

// Settings store
function createSettingsStore() {
  const { subscribe, set, update } = writable<Settings>(defaultSettings);

  return {
    subscribe,
    get: () => get({ subscribe }),

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

    async updateSetting(key: keyof Settings, value: any) {
      try {
        const valueStr = JSON.stringify(value);
        await invoke('update_setting', { key, value: valueStr });
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
