<script lang="ts">
  import { showSettings } from "$lib/stores/ui";
  import { settings } from "$lib/stores/settings";
  import { showToast } from "$lib/stores/ui";
  import { invoke } from "@tauri-apps/api/core";
  import type { ClipboardMode } from "$lib/types";

  let closeAfterSelection = true;
  let clipboardMode: ClipboardMode = "file";
  let hotkey = "";
  let showAds = true;
  let currentSettings: any = null;
  let isSaving = false;

  settings.subscribe(($settings) => {
    if ($settings) {
      currentSettings = $settings;
      closeAfterSelection = $settings.close_after_selection ?? true;
      clipboardMode = $settings.clipboard_mode || "file";
      hotkey = $settings.hotkey || "Cmd+G";
      showAds = $settings.show_ads ?? true;
    }
  });

  function closeModal() {
    showSettings.set(false);
  }

  async function saveSettings() {
    isSaving = true;

    try {
      const newSettings = {
        ...currentSettings,
        close_after_selection: closeAfterSelection,
        clipboard_mode: clipboardMode,
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
</script>

<svelte:window on:keydown={handleKeyDown} />

<div
  class="settings-overlay"
  on:click={handleBackdropClick}
  role="presentation"
>
  <div class="settings-modal">
    <div class="settings-header">
      <h2>Settings</h2>
      <button class="close-btn" on:click={closeModal} aria-label="Close"
        >×</button
      >
    </div>

    <div class="settings-content">
      <div class="setting-group">
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={closeAfterSelection} />
          <span>Close window after copying</span>
        </label>
      </div>

      <div class="setting-group">
        <label for="clipboard-mode">Clipboard Mode</label>
        <select id="clipboard-mode" bind:value={clipboardMode}>
          <option value="file">Copy GIF File</option>
          <option value="url">Copy URL only</option>
        </select>
      </div>

      <div class="setting-group">
        <label for="hotkey">Global Hotkey</label>
        <input
          id="hotkey"
          type="text"
          placeholder="e.g., Cmd+G"
          bind:value={hotkey}
        />
      </div>

      <div class="setting-group support">
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={showAds} />
          <span>Support development (show ads)</span>
        </label>
        <span class="setting-hint">Subtle inline ads from Klipy</span>
      </div>

      <div class="attribution">
        GIFs powered by <a href="https://klipy.com" target="_blank">Klipy</a>
      </div>
    </div>

    <div class="settings-footer">
      <button class="btn secondary" on:click={closeModal} disabled={isSaving}
        >Cancel</button
      >
      <button class="btn primary" on:click={saveSettings} disabled={isSaving}>
        {isSaving ? "Saving..." : "Save"}
      </button>
    </div>
  </div>
</div>

<style>
  .settings-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(4px);
  }

  .settings-modal {
    background: var(--bg-secondary);
    border-radius: 8px;
    width: 90%;
    max-width: 400px;
    max-height: 90vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px;
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

  .close-btn:hover {
    color: var(--text-primary);
  }

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

  .setting-group select,
  .setting-group input[type="text"] {
    padding: 8px 10px;
    font-size: 13px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .setting-group select:focus,
  .setting-group input[type="text"]:focus {
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
