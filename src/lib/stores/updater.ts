import { writable, get } from 'svelte/store';
import { check, type Update } from '@tauri-apps/plugin-updater';

interface UpdaterState {
  update: Update | null;
  checking: boolean;
  error: string | null;
  showDialog: boolean;
  lastChecked: number | null;
}

const initial: UpdaterState = {
  update: null,
  checking: false,
  error: null,
  showDialog: false,
  lastChecked: null,
};

function friendlyError(err: unknown): string {
  const msg = String(err);
  if (msg.includes('Could not fetch') || msg.includes('latest.json')) {
    return 'No update info published yet. Updates will work once a new release is built.';
  }
  if (msg.includes('network') || msg.includes('fetch')) {
    return 'Could not reach the update server. Check your internet connection.';
  }
  return msg;
}

function createUpdater() {
  const store = writable<UpdaterState>(initial);

  return {
    subscribe: store.subscribe,

    /// Run the update check. `silent: true` is used by the auto-check on launch —
    /// it swallows errors (no toast/dialog) so a missing `latest.json` doesn't
    /// nag the user before any release has been published.
    async checkForUpdate(opts?: { silent?: boolean }) {
      if (get(store).checking) return;
      // Dev mode has no updater bundle to compare against, so silent checks
      // would always error. Manual (non-silent) checks still run for testing.
      if (import.meta.env.DEV && opts?.silent) return;

      store.update((s) => ({ ...s, checking: true, error: null }));
      try {
        const result = await check();
        store.update((s) => ({
          ...s,
          update: result ?? null,
          showDialog: !!result,
          lastChecked: Date.now(),
        }));
      } catch (err) {
        if (!opts?.silent) {
          store.update((s) => ({ ...s, error: friendlyError(err) }));
        }
      } finally {
        store.update((s) => ({ ...s, checking: false }));
      }
    },

    setShowDialog(show: boolean) {
      store.update((s) => ({ ...s, showDialog: show }));
    },

    dismiss() {
      store.update((s) => ({ ...s, showDialog: false }));
    },
  };
}

export const updater = createUpdater();
