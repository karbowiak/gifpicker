<script lang="ts">
  import { updater } from "$lib/stores/updater";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  let installing = false;
  let done = false;
  let downloaded = 0;
  let total: number | null = null;

  $: update = $updater.update;
  $: open = $updater.showDialog && !!update;
  $: pct = total ? Math.round((downloaded / total) * 100) : null;

  async function handleInstall() {
    if (!update) return;
    installing = true;
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          total = event.data.contentLength ?? null;
          downloaded = 0;
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
        }
      });
      // Windows installer (passive mode) restarts the app itself.
      // On macOS/Linux the user has to relaunch, so we show a "done" state.
      done = true;
    } catch (err) {
      console.error("Update install failed:", err);
      installing = false;
    }
  }

  function dismiss() {
    updater.dismiss();
    installing = false;
    done = false;
    downloaded = 0;
    total = null;
  }

  function handleBackdrop(event: MouseEvent) {
    if (event.target === event.currentTarget && !installing) dismiss();
  }
</script>

{#if open && update}
  <div
    class="overlay"
    on:click={handleBackdrop}
    transition:fade={{ duration: 150 }}
    role="presentation"
  >
    <div
      class="dialog"
      transition:fly={{ y: 10, duration: 200, easing: cubicOut }}
      role="dialog"
      aria-labelledby="update-title"
    >
      <div class="header">
        <h2 id="update-title">Update available</h2>
        {#if !installing}
          <button class="close-btn" on:click={dismiss} aria-label="Close">×</button>
        {/if}
      </div>

      {#if done}
        <div class="done">
          <div class="check">✓</div>
          <p class="done-title">Update installed</p>
          <p class="done-sub">Restart GIF Picker to use v{update.version}.</p>
          <div class="actions">
            <button class="btn primary" on:click={dismiss}>Done</button>
          </div>
        </div>
      {:else}
        <p class="lead">
          GIF Picker <span class="version">v{update.version}</span> is ready to install.
        </p>
        {#if update.body}
          <p class="notes">{update.body}</p>
        {/if}

        {#if installing}
          <div class="progress-wrap">
            <div class="progress-track">
              <div
                class="progress-bar"
                style="width: {pct != null ? `${pct}%` : '100%'}"
                class:indeterminate={pct == null}
              ></div>
            </div>
            <p class="progress-label">
              {pct != null ? `Downloading… ${pct}%` : "Downloading…"}
            </p>
          </div>
        {:else}
          <div class="actions">
            <button class="btn secondary" on:click={dismiss}>Later</button>
            <button class="btn primary" on:click={handleInstall}>Update & Restart</button>
          </div>
        {/if}
      {/if}
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    backdrop-filter: blur(3px);
    z-index: 1100;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .dialog {
    width: 360px;
    max-width: 92vw;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 10px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
    padding: 18px 20px;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }

  .header h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 22px;
    line-height: 1;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 0;
  }
  .close-btn:hover { color: var(--text-primary); }

  .lead {
    margin: 0 0 10px 0;
    font-size: 13px;
    color: var(--text-secondary);
  }
  .version {
    color: var(--accent-color);
    font-weight: 600;
  }

  .notes {
    margin: 0 0 14px 0;
    padding: 8px 10px;
    max-height: 110px;
    overflow-y: auto;
    font-size: 12px;
    color: var(--text-tertiary);
    background: var(--bg-primary);
    border-radius: 6px;
    white-space: pre-wrap;
  }

  .progress-wrap { margin: 4px 0 12px 0; }
  .progress-track {
    width: 100%;
    height: 6px;
    background: var(--bg-tertiary);
    border-radius: 3px;
    overflow: hidden;
  }
  .progress-bar {
    height: 100%;
    background: var(--accent-color);
    border-radius: 3px;
    transition: width 0.2s ease;
  }
  .progress-bar.indeterminate {
    width: 40% !important;
    animation: slide 1.2s ease-in-out infinite;
  }
  @keyframes slide {
    0%   { transform: translateX(-100%); }
    100% { transform: translateX(250%); }
  }
  .progress-label {
    margin: 6px 0 0 0;
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 6px;
  }

  .btn {
    padding: 7px 14px;
    font-size: 13px;
    font-weight: 500;
    border-radius: 5px;
    border: none;
    cursor: pointer;
    transition: filter 0.15s ease, background 0.15s ease;
  }
  .btn.secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }
  .btn.secondary:hover { background: var(--border-color); }
  .btn.primary {
    background: var(--accent-color);
    color: white;
  }
  .btn.primary:hover { filter: brightness(1.1); }

  .done {
    text-align: center;
    padding: 4px 0;
  }
  .check {
    font-size: 36px;
    color: var(--accent-color);
    margin-bottom: 6px;
  }
  .done-title {
    margin: 0;
    font-weight: 600;
    color: var(--text-primary);
  }
  .done-sub {
    margin: 4px 0 14px 0;
    font-size: 12px;
    color: var(--text-tertiary);
  }
</style>
