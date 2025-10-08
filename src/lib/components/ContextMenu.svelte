<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { contextMenu, closeContextMenu } from '$lib/stores/ui';
  import { isFavorite } from '$lib/types';
  import { invoke } from '@tauri-apps/api/core';
  import { showToast } from '$lib/stores/ui';
  import { favorites } from '$lib/stores/favorites';
  import type { Favorite, GiphyGifResult } from '$lib/types';

  $: show = $contextMenu.show;
  $: x = $contextMenu.x;
  $: y = $contextMenu.y;
  $: item = $contextMenu.item;

  $: isLocal = item && isFavorite(item);

  async function handleCopyToClipboard() {
    if (!item) return;

    try {
      if (isLocal) {
        await invoke('copy_image_to_clipboard', {
          filePath: (item as Favorite).filepath
        });
        showToast('Copied to clipboard!', 'success');
      } else {
        const giphy = item as GiphyGifResult;
        showToast('Downloading GIF...', 'info');
        const downloaded = await invoke<Favorite>('download_giphy_gif', {
          giphyId: giphy.id,
          gifUrl: giphy.gif_url,
          title: giphy.title,
          width: giphy.width,
          height: giphy.height
        });

        await invoke('copy_image_to_clipboard', {
          filePath: downloaded.filepath
        });
        showToast('Copied to clipboard!', 'success');
      }
    } catch (error) {
      console.error('Failed to copy:', error);
      showToast('Failed to copy to clipboard', 'error');
    } finally {
      closeContextMenu();
    }
  }

  async function handleAddToFavorites() {
    if (!item || isLocal) return;

    try {
      const giphy = item as GiphyGifResult;
      showToast('Downloading GIF...', 'info');
      await invoke<Favorite>('download_giphy_gif', {
        giphyId: giphy.id,
        gifUrl: giphy.gif_url,
        title: giphy.title,
        width: giphy.width,
        height: giphy.height
      });
      showToast('Added to favorites!', 'success');
    } catch (error) {
      console.error('Failed to add to favorites:', error);
      showToast('Failed to add to favorites', 'error');
    } finally {
      closeContextMenu();
    }
  }

  async function handleRemoveFromFavorites() {
    if (!item || !isLocal) return;

    try {
      const fav = item as Favorite;
      if (fav.id) {
        await favorites.delete(fav.id);
        showToast('Removed from favorites!', 'success');
      }
    } catch (error) {
      console.error('Failed to remove:', error);
      showToast('Failed to remove from favorites', 'error');
    } finally {
      closeContextMenu();
    }
  }

  async function handleDeleteFile() {
    if (!item || !isLocal) return;

    const confirmed = confirm('Are you sure you want to delete this file permanently?');
    if (!confirmed) {
      closeContextMenu();
      return;
    }

    try {
      const fav = item as Favorite;
      if (fav.id) {
        await favorites.delete(fav.id);
        showToast('File deleted!', 'success');
      }
    } catch (error) {
      console.error('Failed to delete:', error);
      showToast('Failed to delete file', 'error');
    } finally {
      closeContextMenu();
    }
  }

  async function handleViewOnGiphy() {
    if (!item) return;

    try {
      const url = isLocal
        ? (item as Favorite).source_url
        : `https://giphy.com/gifs/${(item as GiphyGifResult).id}`;

      if (url) {
        await invoke('open_url', { url });
      }
    } catch (error) {
      console.error('Failed to open URL:', error);
      showToast('Failed to open URL', 'error');
    } finally {
      closeContextMenu();
    }
  }

  // Close on click outside
  function handleBackdropClick() {
    closeContextMenu();
  }
</script>

{#if show}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    class="context-menu-backdrop"
    on:click={handleBackdropClick}
    transition:fade="{{ duration: 150 }}"
  >
    <div
      class="context-menu"
      style="left: {x}px; top: {y}px;"
      transition:scale="{{ duration: 150, start: 0.95 }}"
      on:click|stopPropagation
    >
      <button class="menu-item" on:click={handleCopyToClipboard}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M5.5 2A1.5 1.5 0 0 0 4 3.5v9A1.5 1.5 0 0 0 5.5 14h5a1.5 1.5 0 0 0 1.5-1.5v-9A1.5 1.5 0 0 0 10.5 2h-5zm0 1h5a.5.5 0 0 1 .5.5v9a.5.5 0 0 1-.5.5h-5a.5.5 0 0 1-.5-.5v-9a.5.5 0 0 1 .5-.5z" fill="currentColor"/>
        </svg>
        Copy to Clipboard
      </button>

      {#if !isLocal}
        <button class="menu-item" on:click={handleAddToFavorites}>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M8 2l2.5 5 5.5.75-4 3.75 1 5.5L8 14l-5 3-1-5.5-4-3.75L3.5 7z" fill="currentColor"/>
          </svg>
          Add to Favorites
        </button>
      {/if}

      {#if isLocal}
        <button class="menu-item" on:click={handleRemoveFromFavorites}>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M8 2l2.5 5 5.5.75-4 3.75 1 5.5L8 14l-5 3-1-5.5-4-3.75L3.5 7z" stroke="currentColor" fill="none"/>
          </svg>
          Remove from Favorites
        </button>

        <div class="menu-divider"></div>

        <button class="menu-item danger" on:click={handleDeleteFile}>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M5.5 5.5A.5.5 0 0 1 6 6v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm2.5 0a.5.5 0 0 1 .5.5v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm3 .5a.5.5 0 0 0-1 0v6a.5.5 0 0 0 1 0V6z" fill="currentColor"/>
            <path fill-rule="evenodd" d="M14.5 3a1 1 0 0 1-1 1H13v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V4h-.5a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1H6a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1h3.5a1 1 0 0 1 1 1v1z" fill="currentColor"/>
          </svg>
          Delete File
        </button>
      {/if}

      {#if (isLocal && (item as Favorite).source_url) || !isLocal}
        <div class="menu-divider"></div>

        <button class="menu-item" on:click={handleViewOnGiphy}>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M0 8a8 8 0 1 1 16 0A8 8 0 0 1 0 8zm7.5-6.923c-.67.204-1.335.82-1.887 1.855A7.97 7.97 0 0 0 5.145 4H7.5V1.077zM4.09 4a9.267 9.267 0 0 1 .64-1.539 6.7 6.7 0 0 1 .597-.933A7.025 7.025 0 0 0 2.255 4H4.09z" fill="currentColor"/>
          </svg>
          View on Giphy
        </button>
      {/if}
    </div>
  </div>
{/if}

<style>
  .context-menu-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 1000;
    background: transparent;
  }

  .context-menu {
    position: fixed;
    min-width: 200px;
    background: var(--bg-primary, #ffffff);
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    padding: 4px;
    z-index: 1001;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 10px 12px;
    border: none;
    background: transparent;
    color: var(--text-primary, #111827);
    font-size: 14px;
    text-align: left;
    cursor: pointer;
    border-radius: 4px;
    transition: background 0.15s ease;
  }

  .menu-item:hover {
    background: var(--bg-secondary, #f9fafb);
  }

  .menu-item.danger {
    color: #ef4444;
  }

  .menu-item.danger:hover {
    background: #fef2f2;
  }

  .menu-divider {
    height: 1px;
    background: var(--border-color, #e5e7eb);
    margin: 4px 0;
  }
</style>
