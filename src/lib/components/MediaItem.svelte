<script lang="ts">
  import type { Favorite, KlipyGifResult } from "$lib/types";
  import { isFavorite } from "$lib/types";
  import { openContextMenu, showToast } from "$lib/stores/ui";
  import { settings } from "$lib/stores/settings";
  import { favorites } from "$lib/stores/favorites";
  import { invoke } from "@tauri-apps/api/core";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { copyItem } from "$lib/utils/copyMedia";
  import { startDrag } from "@crabnebula/tauri-plugin-drag";

  export let item: Favorite | KlipyGifResult;
  // index is forwarded by the parent for keying; not read here
  export const index: number = 0;
  export let selected: boolean = false;
  /// Fired only after a successful copy. The parent uses this to run
  /// close-after-selection, increment usage, etc. — never for the copy itself.
  export let onClick: (item: Favorite | KlipyGifResult) => void = () => {};
  export let onHover: () => void = () => {};
  export let onLeave: () => void = () => {};

  let isLoading = false;
  let imageLoaded = false;
  let imageError = false;
  let imageUrl = "";
  let imageElement: HTMLImageElement | HTMLVideoElement;
  let containerElement: HTMLDivElement;
  let observer: IntersectionObserver;
  let hasLoadedOnce = false;
  let isVisible = false;
  // Briefly true after a successful copy so we can paint the on-tile checkmark.
  // Cleared by a timer rather than per-frame state so the animation finishes
  // even if the parent re-renders the tile mid-pulse.
  let copied = false;
  let copiedTimer: ReturnType<typeof setTimeout> | undefined;

  const dataUrlCache = new Map<string, string>();

  // Grid row span calculation - each row is 10px, add 1 for gap compensation
  const ROW_HEIGHT = 10;
  $: rowSpan = Math.ceil((getBaseHeight() / ROW_HEIGHT)) + 1;

  function getBaseHeight(): number {
    // Target column width ~150px, calculate height from aspect ratio
    const targetWidth = 150;
    const ratio = getAspectRatio(item);
    return targetWidth / ratio;
  }

  $: isLocalFavorite = isFavorite(item);

  // Check if this Klipy result is already in our favorites
  $: matchedFavorite =
    !isLocalFavorite && (item as KlipyGifResult).slug
      ? $favorites.find((f) => f.source_id === (item as KlipyGifResult).slug)
      : undefined;

  // Track favorite state locally to update UI immediately on click
  let isFavoritedLocally = false;
  $: isFavorited = isLocalFavorite || !!matchedFavorite || isFavoritedLocally;

  $: aspectRatio = getAspectRatio(item);
  $: title = isLocalFavorite
    ? (item as Favorite).description || (item as Favorite).filename
    : (item as KlipyGifResult).title || "Untitled";

  // True when the rendered media is an MP4 — used to gate hover-to-play
  // (we can't pause animated GIFs without transcoding them).
  $: isVideoTile = isLocalFavorite
    ? !!(item as Favorite).mp4_filepath
    : !!(item as KlipyGifResult).mp4_url;

  // Item has an MP4 source available at all (independent of what we render).
  // Drives the format badge when the user has chosen MP4 clipboard format.
  $: hasMp4 = isLocalFavorite
    ? !!(item as Favorite).mp4_filepath
    : !!(item as KlipyGifResult).mp4_url;

  // Badge only appears when the user has explicitly chosen MP4 — for the
  // default (GIF) we stay silent. When MP4 is chosen but unavailable, surface
  // "GIF" so the user knows that tile is silently falling back.
  $: formatBadge =
    ($settings?.clipboard_format ?? "gif") === "mp4"
      ? hasMp4 ? "MP4" : "GIF"
      : null;

  function getAspectRatio(item: Favorite | KlipyGifResult): number {
    const w = isLocalFavorite
      ? (item as Favorite).width
      : (item as KlipyGifResult).width;
    const h = isLocalFavorite
      ? (item as Favorite).height
      : (item as KlipyGifResult).height;
    if (w && h && h > 0) return w / h;
    return 1.33;
  }

  async function loadImageUrl() {
    if (hasLoadedOnce) return;
    hasLoadedOnce = true;

    if (isLocalFavorite) {
      const favorite = item as Favorite;
      // Use MP4 for display if available (better performance)
      if (favorite.mp4_filepath) {
        try {
          imageUrl = convertFileSrc(favorite.mp4_filepath);
          return;
        } catch (e) {}
      }
      if (favorite.filepath) {
        if (dataUrlCache.has(favorite.filepath)) {
          imageUrl = dataUrlCache.get(favorite.filepath)!;
          return;
        }
        try {
          imageUrl = convertFileSrc(favorite.filepath);
          dataUrlCache.set(favorite.filepath, imageUrl);
        } catch (e) {
          if (favorite.gif_url) imageUrl = favorite.gif_url;
          else imageError = true;
        }
      } else if (favorite.gif_url) {
        imageUrl = favorite.gif_url;
      } else {
        imageError = true;
      }
    } else {
      const klipyResult = item as KlipyGifResult;
      // Use MP4 for display if available (better performance)
      imageUrl = klipyResult.mp4_url || klipyResult.gif_url;
    }
  }

  onMount(() => {
    // Get the scroll container for proper intersection detection
    const scrollContainer = containerElement?.closest(".masonry-layout");

    observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          isVisible = entry.isIntersecting;
          if (entry.isIntersecting && !hasLoadedOnce) loadImageUrl();

          // Always-playing in-view MP4s feel livelier than hover-to-play and
          // the cost is small because the offscreen branch parks them.
          if (imageElement instanceof HTMLVideoElement) {
            if (entry.isIntersecting) {
              imageElement.play().catch(() => {});
            } else {
              imageElement.pause();
            }
          }
        });
      },
      { root: scrollContainer, rootMargin: "100px", threshold: 0.01 },
    );
    if (containerElement) observer.observe(containerElement);
  });

  onDestroy(() => {
    if (observer && containerElement) {
      observer.unobserve(containerElement);
      observer.disconnect();
    }
    if (copiedTimer) clearTimeout(copiedTimer);
  });


  function handleImageLoad() {
    imageLoaded = true;
  }
  function handleImageError() {
    imageError = true;
  }

  async function handleClick() {
    if (isLoading || copied) return;
    isLoading = true;

    try {
      const mode = $settings?.clipboard_mode ?? "file";
      const format = $settings?.clipboard_format ?? "gif";
      const result = await copyItem(item, mode, format);

      if (!result.ok) {
        // Errors keep the toast — they need attention and a clear message.
        showToast(result.reason, "error");
        return;
      }

      // Success: paint the on-tile checkmark and hand off to the parent for
      // any post-copy housekeeping (close-after-selection, usage tracking).
      copied = true;
      if (copiedTimer) clearTimeout(copiedTimer);
      copiedTimer = setTimeout(() => (copied = false), 750);

      if (result.via === "url") {
        // URL copy is unusual enough that a toast is still warranted —
        // the user probably wants confirmation it wasn't a file copy.
        showToast("Copied URL!", "success");
      }

      onClick(item);
    } catch (error) {
      console.error("Failed to copy:", error);
      showToast("Failed to copy", "error");
    } finally {
      isLoading = false;
    }
  }

  async function handleFavorite(event: MouseEvent) {
    event.stopPropagation();
    if (isFavorited) return;

    const klipyResult = item as KlipyGifResult;
    try {
      await invoke("add_klipy_favorite", {
        gifUrl: klipyResult.gif_url,
        mp4Url: klipyResult.mp4_url || null,
        sourceId: klipyResult.slug,
        sourceUrl: klipyResult.url,
        title: klipyResult.title || "Untitled",
        width: klipyResult.width,
        height: klipyResult.height,
      });
      isFavoritedLocally = true;
      showToast("Added to favorites!", "success");
    } catch (e) {
      console.error("Failed to add to favorites:", e);
      showToast("Failed to add", "error");
    }
  }

  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();
    openContextMenu(event.clientX, event.clientY, item);
  }

  // Drag-out only fires for local favorites with an actual filepath. Klipy
  // results don't have one (the file lives behind a URL), so we'd need to
  // download first — but dragstart is synchronous so we can't. Users can
  // favorite the gif first to enable drag-out.
  $: draggablePath = isLocalFavorite
    ? (item as Favorite).filepath ?? null
    : null;

  async function handleDragStart(event: DragEvent) {
    if (!draggablePath) return;
    // Prevent the native image/video drag (which would attach the data URL or
    // nothing useful) — we replace it with an OS-level file drag via the plugin.
    event.preventDefault();
    try {
      await startDrag({ item: [draggablePath], icon: draggablePath });
    } catch (error) {
      console.error("Failed to start drag:", error);
    }
  }
</script>

<div
  bind:this={containerElement}
  class="media-item"
  class:selected
  class:loading={isLoading}
  class:copied
  class:draggable={!!draggablePath}
  style="--aspect: {aspectRatio}; grid-row: span {rowSpan};"
  draggable={!!draggablePath}
  on:dragstart={handleDragStart}
  on:click={handleClick}
  on:contextmenu={handleContextMenu}
  on:mouseenter={onHover}
  on:mouseleave={onLeave}
  role="button"
  tabindex="0"
  on:keydown={(e) => e.key === "Enter" && handleClick()}
>
  <div class="media-wrapper">
    {#if imageError}
      <div class="error-placeholder">⚠️</div>
    {:else if imageUrl}
      {#if isVideoTile}
        <!-- Visible MP4s loop continuously; the IntersectionObserver above
             pauses anything that scrolls offscreen so we're not decoding
             dozens of streams at once. -->
        <video
          bind:this={imageElement}
          src={imageUrl}
          class="media"
          class:loaded={imageLoaded}
          autoplay
          loop
          muted
          playsinline
          draggable="false"
          on:loadeddata={handleImageLoad}
          on:error={handleImageError}
        ></video>
      {:else}
        <img
          bind:this={imageElement}
          src={imageUrl}
          alt=""
          class="media"
          class:loaded={imageLoaded}
          draggable="false"
          on:load={handleImageLoad}
          on:error={handleImageError}
        />
      {/if}
    {:else}
      <div class="skeleton"></div>
    {/if}

    {#if isLoading}
      <div class="loading-overlay">
        <div class="spinner"></div>
      </div>
    {/if}

    {#if copied}
      <!-- Local success state: green tint + scaling checkmark. Replaces the
           top-of-window toast for the routine copy case so the user's eye
           never has to leave the tile they just clicked. -->
      <div class="copied-overlay" aria-hidden="true">
        <svg width="44" height="44" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="11" fill="rgba(59, 165, 92, 0.95)" />
          <path
            d="M7 12.5l3.5 3.5L17 9"
            stroke="white"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </div>
    {/if}

    <!-- Favorite button (top-right) -->
    <button
      class="favorite-btn"
      class:is-favorite={isFavorited}
      class:visible={isFavorited}
      on:click={handleFavorite}
      aria-label={isFavorited ? "Favorited" : "Add to favorites"}
    >
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill={isFavorited ? "currentColor" : "none"}
      >
        <path
          d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"
          stroke="currentColor"
          stroke-width="2"
        />
      </svg>
    </button>

    {#if formatBadge}
      <!-- Surfaces "what will be copied" when the user has chosen MP4. We
           stay silent for the GIF default so the chrome stays minimal. -->
      <span class="format-badge" class:fallback={formatBadge === "GIF"}>
        {formatBadge}
      </span>
    {/if}

    <!-- Hover overlay with title -->
    <div class="hover-overlay">
      <div class="overlay-info">
        <span class="overlay-title">{title}</span>
      </div>
    </div>
  </div>
</div>

<style>
  .media-item {
    position: relative;
    cursor: pointer;
    border-radius: 4px;
    overflow: hidden;
    background: var(--bg-secondary);
    transition: box-shadow 0.15s ease, transform 0.15s ease, z-index 0s;
  }

  .media-item:hover {
    box-shadow: 0 0 0 2px var(--accent-color);
  }

  .media-item.draggable {
    /* Tiny visual hint that local favorites can be dragged to Discord/Finder.
       Only kicks in on hover so the grid stays calm at rest. */
    cursor: grab;
  }
  .media-item.draggable:active {
    cursor: grabbing;
  }

  .media-item.selected {
    box-shadow: 0 0 0 3px var(--accent-color), 0 8px 24px rgba(0, 0, 0, 0.4);
    transform: scale(1.05);
    z-index: 10;
  }

  .media-item.loading {
    opacity: 0.6;
    pointer-events: none;
  }

  .media-wrapper {
    position: relative;
    width: 100%;
    height: 100%;
  }

  .media {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
    opacity: 0;
    transition: opacity 0.2s ease;
  }

  .media.loaded {
    opacity: 1;
  }

  .skeleton {
    width: 100%;
    height: 100%;
    background: linear-gradient(
      90deg,
      var(--bg-secondary) 25%,
      var(--bg-tertiary) 50%,
      var(--bg-secondary) 75%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
  }

  @keyframes shimmer {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }

  .error-placeholder {
    width: 100%;
    padding: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 24px;
  }

  .loading-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* Hover overlay */
  .hover-overlay {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    background: linear-gradient(
      to top,
      rgba(0, 0, 0, 0.8) 0%,
      transparent 100%
    );
    padding: 24px 8px 8px;
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .media-item:hover .hover-overlay,
  .media-item.selected .hover-overlay {
    opacity: 1;
  }

  .overlay-info {
    flex: 1;
    min-width: 0;
  }

  .overlay-title {
    display: block;
    font-size: 11px;
    font-weight: 500;
    color: white;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .favorite-btn {
    position: absolute;
    top: 6px;
    right: 6px;
    background: rgba(0, 0, 0, 0.4);
    border: none;
    border-radius: 4px;
    padding: 6px;
    cursor: pointer;
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
    opacity: 0;
    z-index: 10;
  }

  .media-item:hover .favorite-btn,
  .favorite-btn.visible {
    opacity: 1;
  }

  .favorite-btn:hover {
    background: rgba(255, 255, 255, 0.3);
    transform: scale(1.1);
  }

  .favorite-btn.is-favorite {
    color: #ff6b6b;
  }

  .favorite-btn.is-favorite svg {
    fill: currentColor;
  }

  .copied-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(59, 165, 92, 0.18);
    pointer-events: none;
    animation: copied-pulse 0.55s ease-out forwards;
    z-index: 11;
  }

  .copied-overlay svg {
    animation: copied-pop 0.45s cubic-bezier(0.34, 1.56, 0.64, 1) forwards;
  }

  @keyframes copied-pulse {
    0%   { opacity: 0; }
    20%  { opacity: 1; }
    100% { opacity: 0; }
  }

  @keyframes copied-pop {
    0%   { transform: scale(0.5); opacity: 0; }
    40%  { transform: scale(1.15); opacity: 1; }
    100% { transform: scale(1.0); opacity: 1; }
  }

  .media-item.copied {
    box-shadow: 0 0 0 3px var(--success-color, #3ba55c);
  }

  .format-badge {
    position: absolute;
    bottom: 6px;
    left: 6px;
    padding: 2px 6px;
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.5px;
    color: white;
    background: rgba(0, 0, 0, 0.6);
    border-radius: 3px;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    pointer-events: none;
    z-index: 5;
  }

  /* "GIF" badge appears only when the user asked for MP4 but the tile only
     has a GIF source — colour it amber so they can spot the fallback. */
  .format-badge.fallback {
    background: rgba(217, 145, 43, 0.85);
  }
</style>
