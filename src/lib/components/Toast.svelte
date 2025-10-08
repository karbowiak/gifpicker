<script lang="ts">
  import { fade, fly } from 'svelte/transition';
  import { toast } from '$lib/stores/ui';

  $: show = $toast.show;
  $: message = $toast.message;
  $: type = $toast.type;

  // Icon for each toast type
  function getIcon(type: 'success' | 'error' | 'info') {
    switch (type) {
      case 'success':
        return `<svg width="20" height="20" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M10 0C4.48 0 0 4.48 0 10s4.48 10 10 10 10-4.48 10-10S15.52 0 10 0zm-2 15l-5-5 1.41-1.41L8 12.17l7.59-7.59L17 6l-9 9z" fill="currentColor"/>
        </svg>`;
      case 'error':
        return `<svg width="20" height="20" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M10 0C4.48 0 0 4.48 0 10s4.48 10 10 10 10-4.48 10-10S15.52 0 10 0zm1 15H9v-2h2v2zm0-4H9V5h2v6z" fill="currentColor"/>
        </svg>`;
      case 'info':
      default:
        return `<svg width="20" height="20" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M10 0C4.48 0 0 4.48 0 10s4.48 10 10 10 10-4.48 10-10S15.52 0 10 0zm1 15H9v-6h2v6zm0-8H9V5h2v2z" fill="currentColor"/>
        </svg>`;
    }
  }
</script>

{#if show}
  <div
    class="toast-container"
    transition:fly="{{ y: -20, duration: 300 }}"
  >
    <div class="toast" class:success={type === 'success'} class:error={type === 'error'} class:info={type === 'info'}>
      <div class="toast-icon">
        {@html getIcon(type)}
      </div>
      <p class="toast-message">{message}</p>
    </div>
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    top: 20px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 9999;
    pointer-events: none;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 20px;
    border-radius: 8px;
    background: var(--bg-primary, #ffffff);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    min-width: 300px;
    max-width: 500px;
  }

  .toast.success {
    background: #10b981;
    color: white;
  }

  .toast.error {
    background: #ef4444;
    color: white;
  }

  .toast.info {
    background: #3b82f6;
    color: white;
  }

  .toast-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .toast-message {
    margin: 0;
    font-size: 14px;
    font-weight: 500;
    line-height: 1.4;
  }
</style>
