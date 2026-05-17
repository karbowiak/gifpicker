<script lang="ts">
  import { showSettings } from "$lib/stores/ui";
  import { settings } from "$lib/stores/settings";
  import { showToast } from "$lib/stores/ui";
  import { invoke } from "@tauri-apps/api/core";
  import { onDestroy } from "svelte";
  import { fly, fade } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import type {
    ClipboardFormat,
    ClipboardMode,
    Settings as AppSettings,
    TileSize,
  } from "$lib/types";
  import HotkeyCapture from "$lib/components/HotkeyCapture.svelte";

  let closeAfterSelection = true;
  let clipboardMode: ClipboardMode = "file";
  let clipboardFormat: ClipboardFormat = "gif";
  let tileSize: TileSize = "medium";
  let alwaysOnTop = false;
  let hotkey = "";
  let showAds = true;
  let currentSettings: AppSettings | null = null;
  let isSaving = false;

  const unsubscribe = settings.subscribe(($settings) => {
    if ($settings) {
      currentSettings = $settings;
      closeAfterSelection = $settings.close_after_selection ?? true;
      clipboardMode = $settings.clipboard_mode || "file";
      clipboardFormat = $settings.clipboard_format || "gif";
      tileSize = $settings.tile_size || "medium";
      alwaysOnTop = $settings.always_on_top ?? false;
      hotkey = $settings.hotkey || "Cmd+G";
      showAds = $settings.show_ads ?? true;
    }
  });

  onDestroy(unsubscribe);

  function closeModal() {
    showSettings.set(false);
  }

  async function saveSettings() {
    if (!currentSettings) return;
    isSaving = true;

    try {
      const newSettings: AppSettings = {
        ...currentSettings,
        close_after_selection: closeAfterSelection,
        clipboard_mode: clipboardMode,
        clipboard_format: clipboardFormat,
        tile_size: tileSize,
        always_on_top: alwaysOnTop,
        hotkey: hotkey.trim(),
        show_ads: showAds,
      };

      await settings.save(newSettings);

      try {
        await invoke("register_hotkey", { hotkey: hotkey.trim() });
      } catch (error) {
        console.error("Failed to register hotkey:", error);
        showToast("Settings saved but hotkey registration failed", "error");
        isSaving = false;
        return;
      }

      try {
        await invoke("set_always_on_top", { value: alwaysOnTop });
      } catch (error) {
        // Non-fatal — the setting is persisted, the window state just won't
        // reflect it until restart. Log but don't fail the save.
        console.error("Failed to apply always-on-top:", error);
      }

      showToast("Settings saved!", "success");
      closeModal();
    } catch (error) {
      console.error("Failed to save settings:", error);
      showToast("Failed to save settings", "error");
    } finally {
      isSaving = false;
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "Escape") closeModal();
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) closeModal();
  }

  // Tauri global shortcuts are intercepted at the OS layer, so if our current
  // hotkey is, say, Cmd+G, pressing Cmd+G to capture a new one would just
  // re-trigger the app rather than reach the capture handler. Unregister
  // everything while recording, then re-register whatever's in `hotkey` after.
  async function suspendGlobalHotkey() {
    try {
      await invoke("unregister_all_hotkeys");
    } catch (error) {
      console.error("Failed to suspend global hotkey:", error);
    }
  }

  async function restoreGlobalHotkey() {
    const target = hotkey.trim();
    if (!target) return;
    try {
      await invoke("register_hotkey", { hotkey: target });
    } catch (error) {
      console.error("Failed to restore global hotkey:", error);
    }
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

<div
  class="settings-overlay"
  on:click={handleBackdropClick}
  transition:fade={{ duration: 150 }}
  role="presentation"
>
  <aside
    class="settings-panel"
    transition:fly={{ x: 360, duration: 220, easing: cubicOut }}
    aria-label="Settings"
  >
    <div class="settings-header">
      <h2>Settings</h2>
      <button class="close-btn" on:click={closeModal} aria-label="Close">×</button>
    </div>

    <div class="settings-content">
      <div class="setting-group">
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={closeAfterSelection} />
          <span>Close window after copying</span>
        </label>
      </div>

      <div class="setting-group">
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={alwaysOnTop} />
          <span>Always on top</span>
        </label>
        <span class="setting-hint">Keep the picker above other windows.</span>
      </div>

      <div class="setting-group">
        <label for="tile-size">Tile Size</label>
        <div class="segmented">
          <button
            type="button"
            class:active={tileSize === "small"}
            on:click={() => (tileSize = "small")}
          >Small</button>
          <button
            type="button"
            class:active={tileSize === "medium"}
            on:click={() => (tileSize = "medium")}
          >Medium</button>
          <button
            type="button"
            class:active={tileSize === "large"}
            on:click={() => (tileSize = "large")}
          >Large</button>
        </div>
        <span class="setting-hint">Smaller tiles = more per screen.</span>
      </div>

      <div class="setting-group">
        <label for="clipboard-mode">Clipboard Mode</label>
        <select id="clipboard-mode" bind:value={clipboardMode}>
          <option value="file">Copy file</option>
          <option value="url">Copy URL only</option>
        </select>
        <span class="setting-hint">What to put on the clipboard when you click a GIF.</span>
      </div>

      {#if clipboardMode === "file"}
        <div class="setting-group">
          <label for="clipboard-format">File Format</label>
          <select id="clipboard-format" bind:value={clipboardFormat}>
            <option value="gif">GIF — best compatibility (Discord, Slack)</option>
            <option value="mp4">MP4 — smaller, smoother, less compatible</option>
          </select>
          <span class="setting-hint">Falls back to GIF if the chosen format isn't available.</span>
        </div>
      {/if}

      <div class="setting-group">
        <label for="hotkey-capture">Global Hotkey</label>
        <HotkeyCapture
          bind:value={hotkey}
          placeholder="Click to record"
          on:capturestart={suspendGlobalHotkey}
          on:captureend={restoreGlobalHotkey}
        />
        <span class="setting-hint">
          Click, then press your shortcut. Needs at least one modifier (Cmd/Ctrl/Alt).
        </span>
      </div>

      <div class="setting-group support">
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={showAds} />
          <span>Support development (show ads)</span>
        </label>
        <span class="setting-hint">Subtle inline ads from Klipy</span>
      </div>

      <div class="attribution">
        GIFs powered by <a href="https://klipy.com" target="_blank" rel="noopener">Klipy</a>
      </div>
    </div>

    <div class="settings-footer">
      <button class="btn secondary" on:click={closeModal} disabled={isSaving}>Cancel</button>
      <button class="btn primary" on:click={saveSettings} disabled={isSaving}>
        {isSaving ? "Saving..." : "Save"}
      </button>
    </div>
  </aside>
</div>

<style>
  .settings-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    backdrop-filter: blur(2px);
    z-index: 1000;
    display: flex;
    justify-content: flex-end;
  }

  .settings-panel {
    background: var(--bg-secondary);
    width: 360px;
    max-width: 95vw;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-shadow: -8px 0 32px rgba(0, 0, 0, 0.35);
    border-left: 1px solid var(--border-color);
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    border-bottom: 1px solid var(--border-color);
  }

  .settings-header h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 24px;
    line-height: 1;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 0;
  }

  .close-btn:hover { color: var(--text-primary); }

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .setting-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .setting-group label {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .setting-group select {
    padding: 8px 10px;
    font-size: 13px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .setting-group select:focus {
    outline: none;
    border-color: var(--accent-color);
  }

  .setting-hint {
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    cursor: pointer;
  }

  .checkbox-label input {
    width: 16px;
    height: 16px;
    accent-color: var(--accent-color);
  }

  .checkbox-label span {
    color: var(--text-primary);
  }

  /* Segmented control for tile size — feels lighter than a select for 3 fixed
     choices and the active state is visible at a glance. */
  .segmented {
    display: flex;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    overflow: hidden;
  }
  .segmented button {
    flex: 1;
    padding: 7px 0;
    background: var(--bg-primary);
    color: var(--text-secondary);
    border: none;
    border-right: 1px solid var(--border-color);
    font-size: 12px;
    cursor: pointer;
    transition: background 0.15s ease, color 0.15s ease;
  }
  .segmented button:last-child { border-right: none; }
  .segmented button:hover { background: var(--bg-tertiary); color: var(--text-primary); }
  .segmented button.active {
    background: var(--accent-color);
    color: white;
  }

  .setting-group.support {
    background: var(--accent-color-light);
    padding: 12px;
    border-radius: 6px;
  }

  .attribution {
    text-align: center;
    font-size: 11px;
    color: var(--text-tertiary);
    padding-top: 8px;
    border-top: 1px solid var(--border-color);
  }

  .attribution a {
    color: var(--accent-color);
    text-decoration: none;
  }

  .settings-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--border-color);
  }

  .btn {
    padding: 8px 16px;
    font-size: 13px;
    font-weight: 500;
    border-radius: 4px;
    cursor: pointer;
    border: none;
    transition: all 0.15s ease;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn.secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .btn.secondary:hover:not(:disabled) {
    background: var(--border-color);
  }

  .btn.primary {
    background: var(--accent-color);
    color: white;
  }

  .btn.primary:hover:not(:disabled) {
    filter: brightness(1.1);
  }
</style>
