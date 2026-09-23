<script lang="ts">
  import { onDestroy } from "svelte";
  import { CircleAlert, X } from "lucide-svelte";

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
  <div class="ka-toast-region">
    <div role="alert" aria-live="assertive" class="ka-toast">
      <CircleAlert class="w-5 h-5 ka-icon" aria-hidden="true" />
      <div>{message}</div>
      <button
        type="button"
        class="ka-button ka-button--ghost ka-icon-button"
        onclick={onDismiss}
        aria-label="Dismiss error"
        title="Dismiss"
      >
        <X class="w-5 h-5" aria-hidden="true" />
      </button>
    </div>
  </div>
{/if}
