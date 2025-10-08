<script lang="ts">
  import { showSettings } from '$lib/stores/ui';
  import { settings } from '$lib/stores/settings';
  import { showToast } from '$lib/stores/ui';

  let giphyApiKey = '';
  let closeAfterSelection = true;
  let currentSettings: any = null;
  let isSaving = false;

  // Load current settings
  settings.subscribe($settings => {
    if ($settings) {
      currentSettings = $settings;
      giphyApiKey = $settings.giphy_api_key || '';
      closeAfterSelection = $settings.close_after_selection ?? true;
    }
  });

  function closeModal() {
    // Don't allow closing if API key is not set
    if (!currentSettings?.giphy_api_key && !giphyApiKey.trim()) {
      showToast('Please add your Giphy API key to continue', 'error');
      return;
    }
    showSettings.set(false);
  }

  async function saveSettings() {
    isSaving = true;

    // Validate API key
    if (!giphyApiKey.trim()) {
      showToast('Giphy API key is required', 'error');
      isSaving = false;
      return;
    }

    try {
      const newSettings = {
        ...currentSettings,
        giphy_api_key: giphyApiKey.trim(),
        close_after_selection: closeAfterSelection
      };

      await settings.save(newSettings);
      showToast('Settings saved successfully!', 'success');
      closeModal();
    } catch (error) {
      console.error('Failed to save settings:', error);
      showToast('Failed to save settings', 'error');
    } finally {
      isSaving = false;
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      closeModal();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      closeModal();
    }
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

<div class="settings-overlay" on:click={handleBackdropClick} role="presentation">
  <div class="settings-modal">
    <div class="settings-header">
      <h2>Settings</h2>
      <button class="close-button" on:click={closeModal} aria-label="Close settings">
        <svg width="20" height="20" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M15 5L5 15M5 5l10 10" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </button>
    </div>

    <div class="settings-content">
      <div class="setting-group">
        <label for="giphy-api-key">
          <span class="setting-label">Giphy API Key <span class="required">*</span></span>
          <span class="setting-description">
            Required to search GIFs.
            <a href="https://developers.giphy.com" target="_blank" rel="noopener noreferrer">
              Get your free API key from Giphy Developers
            </a>
            - Login, create an app, and copy the API key.
          </span>
        </label>
        <input
          id="giphy-api-key"
          type="text"
          class="setting-input"
          placeholder="Enter your Giphy API key..."
          bind:value={giphyApiKey}
          required
        />
      </div>

      <div class="setting-group">
        <label class="checkbox-label">
          <input
            type="checkbox"
            bind:checked={closeAfterSelection}
          />
          <span class="setting-label">Close window after copying</span>
        </label>
        <span class="setting-description">Automatically close the app after selecting a GIF</span>
      </div>
    </div>

    <div class="settings-footer">
      <button class="button button-secondary" on:click={closeModal} disabled={isSaving}>
        Cancel
      </button>
      <button class="button button-primary" on:click={saveSettings} disabled={isSaving}>
        {isSaving ? 'Saving...' : 'Save Settings'}
      </button>
    </div>
  </div>
</div>

<style>
  .settings-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(4px);
  }

  .settings-modal {
    background: var(--bg-primary, #ffffff);
    border-radius: 12px;
    width: 90%;
    max-width: 500px;
    max-height: 90vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px;
    border-bottom: 1px solid var(--border-color, #e5e7eb);
  }

  .settings-header h2 {
    margin: 0;
    font-size: 20px;
    font-weight: 600;
    color: var(--text-primary, #111827);
  }

  .close-button {
    background: none;
    border: none;
    color: var(--text-secondary, #6b7280);
    cursor: pointer;
    padding: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    transition: all 0.2s ease;
  }

  .close-button:hover {
    background: var(--bg-secondary, #f9fafb);
    color: var(--text-primary, #111827);
  }

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }

  .setting-group {
    margin-bottom: 24px;
  }

  .setting-group:last-child {
    margin-bottom: 0;
  }

  .setting-group label {
    display: block;
    margin-bottom: 8px;
  }

  .setting-label {
    display: block;
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary, #111827);
    margin-bottom: 4px;
  }

  .required {
    color: #ef4444;
  }

  .setting-description {
    display: block;
    font-size: 12px;
    color: var(--text-secondary, #6b7280);
    line-height: 1.5;
  }

  .setting-description a {
    color: var(--accent-color, #3b82f6);
    text-decoration: none;
  }

  .setting-description a:hover {
    text-decoration: underline;
  }

  .setting-input {
    width: 100%;
    padding: 10px 12px;
    font-size: 14px;
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 8px;
    background: var(--bg-secondary, #f9fafb);
    color: var(--text-primary, #111827);
    transition: all 0.2s ease;
  }

  .setting-input:focus {
    outline: none;
    border-color: var(--accent-color, #3b82f6);
    background: var(--bg-primary, #ffffff);
    box-shadow: 0 0 0 3px var(--accent-color-light, rgba(59, 130, 246, 0.1));
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 12px;
    cursor: pointer;
  }

  .checkbox-label input[type="checkbox"] {
    width: 18px;
    height: 18px;
    cursor: pointer;
    accent-color: var(--accent-color, #3b82f6);
  }

  .checkbox-label .setting-label {
    margin-bottom: 0;
    cursor: pointer;
  }

  .settings-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 20px 24px;
    border-top: 1px solid var(--border-color, #e5e7eb);
  }

  .button {
    padding: 10px 20px;
    font-size: 14px;
    font-weight: 500;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
    border: none;
  }

  .button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .button-secondary {
    background: var(--bg-secondary, #f9fafb);
    color: var(--text-primary, #111827);
    border: 1px solid var(--border-color, #e5e7eb);
  }

  .button-secondary:hover:not(:disabled) {
    background: var(--bg-tertiary, #f3f4f6);
  }

  .button-primary {
    background: var(--accent-color, #3b82f6);
    color: white;
  }

  .button-primary:hover:not(:disabled) {
    background: var(--accent-color-dark, #2563eb);
  }

  .button:active:not(:disabled) {
    transform: scale(0.98);
  }
</style>
