import { writable } from 'svelte/store';

// Persisted in localStorage rather than the SQLite settings table: this is
// pure UI state, never touched by the backend, and surviving a wipe of the
// app's data dir is fine — these are just chip suggestions.
const STORAGE_KEY = 'gifpicker.recent_searches.v1';
const MAX_RECENT = 12;

function load(): string[] {
  if (typeof localStorage === 'undefined') return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((s): s is string => typeof s === 'string').slice(0, MAX_RECENT);
  } catch {
    return [];
  }
}

function persist(items: string[]) {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(items));
  } catch {
    // Quota / privacy mode — silently degrade to in-memory only.
  }
}

function createRecentSearches() {
  const { subscribe, set, update } = writable<string[]>(load());

  return {
    subscribe,

    /// Record a successful search. Moves duplicates to the front, caps the list.
    record(query: string) {
      const trimmed = query.trim();
      if (!trimmed) return;
      update((items) => {
        const next = [trimmed, ...items.filter((s) => s.toLowerCase() !== trimmed.toLowerCase())]
          .slice(0, MAX_RECENT);
        persist(next);
        return next;
      });
    },

    remove(query: string) {
      update((items) => {
        const next = items.filter((s) => s !== query);
        persist(next);
        return next;
      });
    },

    clear() {
      persist([]);
      set([]);
    },
  };
}

export const recentSearches = createRecentSearches();
