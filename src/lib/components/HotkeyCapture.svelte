<script lang="ts">
  import { createEventDispatcher, onDestroy } from "svelte";
  import {
    displayAccelerator,
    eventToAccelerator,
    isModifierKey,
  } from "$lib/utils/hotkey";

  /// The currently-saved accelerator, e.g. "Option+Cmd+G". Two-way bound so
  /// the parent's value is updated when the user captures a new combo.
  export let value: string = "";
  export let placeholder: string = "Click to set hotkey";

  // Events:
  //   change       — emitted with the new accelerator string on commit
  //   capturestart — emitted when the user enters recording mode; the parent
  //                  MUST unregister any global shortcut that conflicts with
  //                  capture (e.g. the very hotkey we're trying to rebind),
  //                  otherwise the OS swallows the keystroke before we see it.
  //   captureend   — emitted when recording stops (commit, cancel, or blur);
  //                  parent re-registers the appropriate hotkey.
  const dispatch = createEventDispatcher<{
    change: string;
    capturestart: void;
    captureend: { committed: boolean };
  }>();

  let capturing = false;
  // Visible-while-capturing preview of held modifiers, e.g. "Option+Cmd+".
  let preview = "";

  function startCapture() {
    capturing = true;
    preview = "";
    dispatch("capturestart");
  }

  function stopCapture(commit?: string) {
    capturing = false;
    preview = "";
    if (commit !== undefined) {
      value = commit;
      dispatch("change", commit);
    }
    dispatch("captureend", { committed: commit !== undefined });
  }

  function modifierPreview(event: KeyboardEvent): string {
    const parts: string[] = [];
    if (event.ctrlKey) parts.push("Ctrl");
    if (event.altKey) parts.push(isMac ? "Option" : "Alt");
    if (event.shiftKey) parts.push("Shift");
    if (event.metaKey) parts.push(isMac ? "Cmd" : "Meta");
    return parts.length ? parts.join("+") + "+" : "";
  }

  const isMac =
    typeof navigator !== "undefined" &&
    /Mac|iPhone|iPad/.test(navigator.platform);

  function handleKeyDown(event: KeyboardEvent) {
    if (!capturing) return;

    // Always swallow keys while capturing — we don't want Tab to leave focus
    // or Esc to bubble up to anyone's window handler.
    event.preventDefault();
    event.stopPropagation();

    if (event.key === "Escape") {
      stopCapture();
      return;
    }

    // Modifier alone — just update the visible preview, wait for a real key.
    if (isModifierKey(event.code)) {
      preview = modifierPreview(event);
      return;
    }

    const accel = eventToAccelerator(event);
    if (!accel) {
      // Unsupported key (e.g. dead key) — keep waiting.
      return;
    }

    // Require at least one modifier — a bare letter would steal that key
    // globally across the OS, which is almost never what you want.
    const hasModifier =
      event.ctrlKey || event.altKey || event.metaKey ||
      // Shift alone counts only when paired with a real key like F-keys.
      (event.shiftKey && /^F\d/.test(event.code));
    if (!hasModifier) {
      preview = "Need a modifier (Cmd/Ctrl/Alt)";
      return;
    }

    stopCapture(accel);
  }

  function handleKeyUp(event: KeyboardEvent) {
    // Drop the modifier-only preview when the user releases everything.
    if (!capturing) return;
    if (isModifierKey(event.code)) {
      preview = modifierPreview(event);
    }
  }

  function handleBlur() {
    if (capturing) stopCapture();
  }

  // Capture phase so we beat ANY other key listener while recording (page
  // keyboard nav, modal escape handlers, etc.). Without this, hitting Esc
  // during capture would close the Settings modal.
  function attachListeners() {
    window.addEventListener("keydown", handleKeyDown, true);
    window.addEventListener("keyup", handleKeyUp, true);
  }
  function detachListeners() {
    window.removeEventListener("keydown", handleKeyDown, true);
    window.removeEventListener("keyup", handleKeyUp, true);
  }

  $: if (capturing) attachListeners();
  $: if (!capturing) detachListeners();
  onDestroy(() => {
    // If the parent unmounts us mid-capture (e.g. user closes the Settings
    // modal while recording), we still owe them a captureend so they can
    // restore the global hotkey. Otherwise the OS-level shortcut stays dead.
    if (capturing) dispatch("captureend", { committed: false });
    detachListeners();
  });

  $: display = capturing
    ? preview || "Press your hotkey…"
    : value
      ? displayAccelerator(value)
      : "";
</script>

<button
  type="button"
  class="hotkey-button"
  class:capturing
  on:click={startCapture}
  on:blur={handleBlur}
  aria-label="Hotkey"
>
  {#if display}
    <span class="combo">{display}</span>
  {:else}
    <span class="placeholder">{placeholder}</span>
  {/if}
  {#if !capturing && value}
    <span
      class="clear"
      role="button"
      tabindex="0"
      aria-label="Clear hotkey"
      on:click|stopPropagation={() => stopCapture("")}
      on:keydown|stopPropagation={(e) => (e.key === "Enter" || e.key === " ") && stopCapture("")}
    >×</span>
  {/if}
</button>

<style>
  .hotkey-button {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 10px;
    font-size: 13px;
    font-family: inherit;
    text-align: left;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--bg-primary);
    color: var(--text-primary);
    cursor: pointer;
    transition: border-color 0.15s ease, background 0.15s ease;
  }
  .hotkey-button:hover {
    border-color: var(--text-tertiary);
  }
  .hotkey-button:focus {
    outline: none;
    border-color: var(--accent-color);
  }
  .hotkey-button.capturing {
    border-color: var(--accent-color);
    background: var(--accent-color-light);
    /* Subtle pulse so it's obvious we're listening */
    animation: hotkey-pulse 1.2s ease-in-out infinite;
  }
  @keyframes hotkey-pulse {
    0%, 100% { box-shadow: 0 0 0 0 var(--accent-color-light); }
    50%      { box-shadow: 0 0 0 4px var(--accent-color-light); }
  }
  .combo {
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-weight: 600;
    letter-spacing: 0.5px;
  }
  .placeholder {
    color: var(--text-tertiary);
    font-style: italic;
  }
  .clear {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--bg-tertiary);
    color: var(--text-tertiary);
    font-size: 14px;
    line-height: 1;
    cursor: pointer;
    transition: background 0.15s ease, color 0.15s ease;
  }
  .clear:hover {
    background: var(--error-color);
    color: white;
  }
</style>
