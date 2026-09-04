<script lang="ts">
  import { onDestroy } from "svelte";
  import { X } from "lucide-svelte";

  interface Props {
    message: string;
    onDismiss: () => void;
    duration?: number;
  }

  let { message, onDismiss, duration = 4000 }: Props = $props();

  let timeout: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    if (!message) return;
    if (timeout) {
      clearTimeout(timeout);
    }
    timeout = setTimeout(() => {
      onDismiss();
    }, duration);

    return () => {
      if (timeout) {
        clearTimeout(timeout);
        timeout = null;
      }
    };
  });

  onDestroy(() => {
    if (timeout) {
      clearTimeout(timeout);
    }
  });
</script>

{#if message}
  <div class="fixed bottom-4 right-4 z-press-toast max-w-sm">
    <div
      role="alert"
      aria-live="assertive"
      class="bg-press-sunken border border-press-error text-press-text rounded-lg shadow-press-overlay px-4 py-3"
    >
      <div class="flex items-start gap-3">
        <p class="text-press-ui leading-relaxed flex-1">{message}</p>
        <button
          class="text-press-muted hover:text-press-text transition-colors"
          onclick={onDismiss}
          aria-label="Dismiss error"
        >
          <X class="w-4 h-4" />
        </button>
      </div>
    </div>
  </div>
{/if}
