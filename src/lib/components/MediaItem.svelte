<script lang="ts">
  import type { Favorite, GiphyGifResult } from '$lib/types';
  import { isFavorite } from '$lib/types';
  import { openContextMenu } from '$lib/stores/ui';
  import { showToast } from '$lib/stores/ui';
  import { settings } from '$lib/stores/settings';
  import { invoke } from '@tauri-apps/api/core';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';

  export let item: Favorite | GiphyGifResult;
  export let selected: boolean = false;
  export let onClick: (item: Favorite | GiphyGifResult) => void = () => {};

  let isLoading = false;
  let imageLoaded = false;
  let imageError = false;
  let imageUrl = '';
  let imageElement: HTMLImageElement | HTMLVideoElement;
  let containerElement: HTMLDivElement;
  let observer: IntersectionObserver;
  let hasLoadedOnce = false; // Track if we've tried loading
  let isInView = false; // Track if currently in viewport

  // Cache for data URLs to avoid reloading
  const dataUrlCache = new Map<string, string>();

  // Get the appropriate properties based on item type
  $: isLocalFavorite = isFavorite(item);
  $: title = isLocalFavorite
    ? (item as Favorite).filename.replace(/\.[^/.]+$/, '') // Remove extension
    : (item as GiphyGifResult).title || 'Untitled';
  $: metadata = isLocalFavorite
    ? `${(item as Favorite).width || '?'}x${(item as Favorite).height || '?'} • Used ${(item as Favorite).use_count || 0} times`
    : `${(item as GiphyGifResult).width}x${(item as GiphyGifResult).height}`;

  // Load image URL only when visible (lazy loading)
  async function loadImageUrl() {
    if (hasLoadedOnce) return; // Already tried loading, don't retry
    hasLoadedOnce = true;

    if (isLocalFavorite) {
      const favorite = item as Favorite;

      // Prefer MP4 for display if available (much better performance)
      if (favorite.mp4_filepath) {
        try {
          const assetUrl = convertFileSrc(favorite.mp4_filepath);
          imageUrl = assetUrl;
          dataUrlCache.set(favorite.mp4_filepath, assetUrl);
          return;
        } catch (error) {
          console.error('Failed to convert MP4 to asset URL:', error);
          // Fall through to try GIF
        }
      }

      // Fall back to GIF
      if (favorite.filepath) {
        // Check cache first
        if (dataUrlCache.has(favorite.filepath)) {
          imageUrl = dataUrlCache.get(favorite.filepath)!;
          return;
        }

        // Use Tauri's convertFileSrc for proper asset protocol handling
        try {
          const assetUrl = convertFileSrc(favorite.filepath);
          imageUrl = assetUrl;
          dataUrlCache.set(favorite.filepath, assetUrl); // Cache it
        } catch (error) {
          console.error('Failed to convert to asset URL:', error);
          // Fallback to gif_url if available
          if (favorite.gif_url) {
            imageUrl = favorite.gif_url;
          } else {
            imageError = true;
          }
        }
      } else if (favorite.gif_url) {
        // Use gif_url directly
        imageUrl = favorite.gif_url;
      } else {
        imageError = true;
      }
    } else {
      // For Giphy search results, prefer MP4 (better performance), fallback to GIF
      const giphyResult = item as GiphyGifResult;
      imageUrl = giphyResult.mp4_url || giphyResult.gif_url;
    }
  }

  onMount(() => {
    // Set up Intersection Observer for lazy loading and viewport tracking
    observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          isInView = entry.isIntersecting;

          // Load image when coming into view (with buffer)
          if (entry.isIntersecting && !hasLoadedOnce) {
            loadImageUrl();
          }

          // Performance optimization: Hide image when far out of view
          // This helps reduce GPU/CPU usage for GIF rendering
          if (imageElement) {
            if (entry.isIntersecting || entry.intersectionRatio > 0) {
              // In or near viewport - show image
              imageElement.style.visibility = 'visible';
            } else {
              // Far from viewport - hide but keep space (better than display:none)
              imageElement.style.visibility = 'hidden';
            }
          }
        });
      },
      {
        rootMargin: '400px', // Large buffer for smooth experience
        threshold: 0.01
      }
    );

    if (containerElement) {
      observer.observe(containerElement);
    }
  });

  onDestroy(() => {
    if (observer && containerElement) {
      observer.unobserve(containerElement);
      observer.disconnect();
    }
  });

  // Handle image load
  function handleImageLoad() {
    imageLoaded = true;
  }

  // Handle image error
  function handleImageError() {
    imageError = true;
    console.error('Failed to load image:', imageUrl);
  }

  // Handle click - copy to clipboard
  async function handleClick() {
    if (isLoading) return;

    onClick(item);

    isLoading = true;

    try {
      const clipboardMode = $settings?.clipboard_mode || 'file';

      if (isLocalFavorite) {
        const favorite = item as Favorite;

        if (clipboardMode === 'file') {
          // Copy the file itself (preserves GIF animation)
          if (favorite.filepath) {
            await invoke('copy_file_path_to_clipboard', {
              filePath: favorite.filepath
            });
          } else if (favorite.gif_url) {
            // Fallback to URL if no local file
            await invoke('copy_text_to_clipboard', {
              text: favorite.gif_url
            });
          }
        } else {
          // Copy URL mode - always use Giphy URL
          if (favorite.gif_url) {
            await invoke('copy_text_to_clipboard', {
              text: favorite.gif_url
            });
          } else {
            showToast('No Giphy URL available for this GIF', 'error');
            isLoading = false;
            return;
          }
        }

        // Increment use count
        await invoke('increment_use_count', {
          id: favorite.id
        });

        showToast('Copied to clipboard!', 'success');
      } else {
        // For Giphy search results (not favorited yet)
        const giphyResult = item as GiphyGifResult;

        if (clipboardMode === 'file') {
          // Download and copy the file
          showToast('Downloading GIF...', 'info');
          const result = await invoke<{filepath: string}>('download_giphy_gif', {
            gifUrl: giphyResult.gif_url,
            filename: `${giphyResult.id}.gif`
          });

          await invoke('copy_file_path_to_clipboard', {
            filePath: result.filepath
          });
          showToast('GIF copied to clipboard!', 'success');
        } else {
          // Just copy the URL
          await invoke('copy_text_to_clipboard', {
            text: giphyResult.gif_url
          });
          showToast('GIF URL copied to clipboard!', 'success');
        }
      }
    } catch (error) {
      console.error('Failed to copy to clipboard:', error);
      showToast('Failed to copy to clipboard', 'error');
    } finally {
      isLoading = false;
    }
  }

  // Handle adding Giphy GIF to favorites
  async function handleAddToFavorites(event: MouseEvent) {
    event.stopPropagation(); // Prevent click from bubbling to parent

    if (isLocalFavorite) return; // Already a favorite

    const giphyResult = item as GiphyGifResult;

    try {
      showToast('Adding to favorites...', 'info');

      await invoke('add_giphy_favorite', {
        gifUrl: giphyResult.gif_url,
        mp4Url: giphyResult.mp4_url || null, // Pass MP4 URL if available
        sourceId: giphyResult.id,
        sourceUrl: giphyResult.url,
        title: giphyResult.title || 'Untitled',
        width: parseInt(giphyResult.width, 10),
        height: parseInt(giphyResult.height, 10)
      });

      showToast('Added to favorites!', 'success');
    } catch (error) {
      console.error('Failed to add to favorites:', error);
      showToast('Failed to add to favorites', 'error');
    }
  }

  // Handle right-click context menu
  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();
    openContextMenu(event.clientX, event.clientY, item);
  }
</script>

<div
  bind:this={containerElement}
  class="media-item"
  class:selected
  class:loading={isLoading}
  on:click={handleClick}
  on:contextmenu={handleContextMenu}
  role="button"
  tabindex="0"
  on:keydown={(e) => e.key === 'Enter' && handleClick()}
>
  <div class="image-wrapper">
    {#if !imageLoaded && !imageError}
      <div class="skeleton-loader"></div>
    {/if}

    {#if imageError}
      <div class="error-placeholder">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M21 19V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2zM8.5 13.5l2.5 3.01L14.5 12l4.5 6H5l3.5-4.5z" fill="currentColor"/>
        </svg>
        <p>Failed to load</p>
      </div>
    {:else if imageUrl}
      {#if (isLocalFavorite && (item as Favorite).mp4_filepath) || (!isLocalFavorite && (item as GiphyGifResult).mp4_url)}
        <!-- Use video element for MP4 (much better performance) -->
        <video
          bind:this={imageElement}
          src={imageUrl}
          title={title}
          class="media-image"
          class:loaded={imageLoaded}
          autoplay
          loop
          muted
          playsinline
          on:loadeddata={handleImageLoad}
          on:error={handleImageError}
        ></video>
      {:else}
        <!-- Use img element for GIFs (fallback) -->
        <img
          bind:this={imageElement}
          src={imageUrl}
          alt={title}
          class="media-image"
          class:loaded={imageLoaded}
          on:load={handleImageLoad}
          on:error={handleImageError}
        />
      {/if}
    {/if}    {#if isLoading}
      <div class="loading-overlay">
        <div class="spinner"></div>
      </div>
    {/if}

    <div class="overlay">
      <div class="overlay-content">
        <h3 class="title">{title}</h3>
        <p class="metadata">{metadata}</p>
        <div class="badges">
          {#if !isLocalFavorite}
            <span class="badge">Giphy</span>
          {/if}
        </div>
      </div>

      {#if !isLocalFavorite}
        <button
          class="favorite-btn"
          on:click={handleAddToFavorites}
          title="Add to favorites"
          aria-label="Add to favorites"
        >
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .media-item {
    position: relative;
    cursor: pointer;
    border-radius: 8px;
    overflow: hidden;
    background: var(--bg-secondary, #f9fafb);
    transition: all 0.2s ease;
    border: 2px solid transparent;
  }

  .media-item:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .media-item.selected {
    border-color: var(--accent-color, #3b82f6);
    box-shadow: 0 0 0 2px var(--accent-color, #3b82f6);
  }

  .media-item.loading {
    opacity: 0.7;
    pointer-events: none;
  }

  .image-wrapper {
    position: relative;
    width: 100%;
    display: block;
  }

  .skeleton-loader {
    width: 100%;
    padding-bottom: 75%; /* 4:3 aspect ratio placeholder */
    background: linear-gradient(
      90deg,
      var(--bg-secondary, #f9fafb) 25%,
      var(--bg-tertiary, #f3f4f6) 50%,
      var(--bg-secondary, #f9fafb) 75%
    );
    background-size: 200% 100%;
    animation: skeleton-loading 1.5s ease-in-out infinite;
  }

  @keyframes skeleton-loading {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }

  .error-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
    color: var(--text-tertiary, #9ca3af);
    text-align: center;
  }

  .error-placeholder p {
    margin-top: 8px;
    font-size: 12px;
  }

  .media-image {
    display: block;
    width: 100%;
    height: auto;
    opacity: 0;
    transition: opacity 0.3s ease;
  }

  .media-image.loaded {
    opacity: 1;
  }

  .loading-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .overlay {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    background: linear-gradient(to top, rgba(0, 0, 0, 0.8), transparent);
    padding: 16px;
    opacity: 0;
    transition: opacity 0.2s ease;
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
  }

  .media-item:hover .overlay {
    opacity: 1;
  }

  .overlay-content {
    color: white;
    flex: 1;
  }

  .title {
    font-size: 14px;
    font-weight: 600;
    margin: 0 0 4px 0;
    line-height: 1.4;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
  }

  .metadata {
    font-size: 12px;
    margin: 0 0 8px 0;
    opacity: 0.9;
  }

  .badges {
    display: flex;
    gap: 8px;
  }

  .badge {
    display: inline-block;
    padding: 4px 8px;
    background: rgba(59, 130, 246, 0.9);
    border-radius: 4px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .favorite-btn {
    background: rgba(255, 255, 255, 0.2);
    border: 2px solid rgba(255, 255, 255, 0.5);
    border-radius: 50%;
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    color: white;
    transition: all 0.2s ease;
    flex-shrink: 0;
    margin-left: 12px;
  }

  .favorite-btn:hover {
    background: rgba(255, 255, 255, 0.3);
    border-color: white;
    transform: scale(1.1);
  }

  .favorite-btn:active {
    transform: scale(0.95);
  }

  .favorite-btn svg {
    fill: none;
  }

  .favorite-btn:hover svg {
    fill: rgba(239, 68, 68, 0.8);
  }
</style>
