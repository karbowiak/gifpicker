import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Settings } from '$lib/types';

// Default settings
const defaultSettings: Settings = {
  giphy_api_key: undefined,
  hotkey: 'Option+Cmd+G',
  window_width: 800,
  window_height: 600,
  max_item_width: 400,
  close_after_selection: true,
  launch_at_startup: false,
  theme: 'system',
  clipboard_mode: 'file'
};

// Settings store
function createSettingsStore() {
  const { subscribe, set, update } = writable<Settings>(defaultSettings);

  return {
    subscribe,

    // Load settings from backend
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

    // Save all settings
    async save(settings: Settings) {
      try {
        await invoke('save_settings', { settings });
        set(settings);
      } catch (error) {
        console.error('Failed to save settings:', error);
        throw error;
      }
    },

    // Update a single setting
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

    // Reset to defaults
    reset() {
      set(defaultSettings);
    }
  };
}

export const settings = createSettingsStore();
