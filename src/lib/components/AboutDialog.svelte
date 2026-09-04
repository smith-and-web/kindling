<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { X, ExternalLink, Flame, Send } from "lucide-svelte";
  import { onMount } from "svelte";

  let { onClose, onSendFeedback }: { onClose: () => void; onSendFeedback: () => void } = $props();

  let version = $state("...");

  onMount(async () => {
    try {
      version = await getVersion();
    } catch {
      version = "unknown";
    }
  });

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    }
  }

  async function openLink(url: string) {
    await openUrl(url);
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="fixed inset-0 bg-press-overlay flex items-center justify-center z-press-modal p-4"
  role="dialog"
  aria-modal="true"
  aria-labelledby="about-title"
  tabindex="-1"
>
  <div
    class="app-dialog-surface bg-press-surface rounded-xl shadow-press-overlay max-w-sm w-full flex flex-col"
    data-testid="about-dialog"
  >
    <div class="flex items-center justify-between p-5 border-b border-press-border shrink-0">
      <h2 id="about-title" class="text-press-body-lg font-heading font-semibold text-press-text">
        About Kindling
      </h2>
      <button
        onclick={onClose}
        class="p-1 rounded hover:bg-press-sunken text-press-muted transition-colors"
        aria-label="Close"
      >
        <X class="w-5 h-5" />
      </button>
    </div>

    <div class="p-5 flex flex-col items-center text-center gap-4">
      <div class="w-14 h-14 rounded-2xl bg-press-accent-wash flex items-center justify-center">
        <Flame class="w-8 h-8 text-press-accent-text" />
      </div>

      <div>
        <h3 class="text-press-body-lg font-heading font-semibold text-press-text">Kindling</h3>
        <p class="text-press-ui text-press-muted mt-0.5">Version {version}</p>
      </div>

      <p class="font-prose text-press-body text-press-text leading-relaxed max-w-press-measure">
        Spark your draft &mdash; Bridge the gap between outline and prose.
      </p>

      <div class="w-full border-t border-press-border pt-4 flex flex-col gap-2">
        <button
          onclick={onSendFeedback}
          class="flex items-center gap-2 w-full px-3 py-2 text-press-ui text-press-muted hover:text-press-text hover:bg-press-sunken rounded-lg transition-colors"
        >
          <Send class="w-4 h-4 shrink-0" />
          Send Feedback
        </button>
        <button
          onclick={() => openLink("https://github.com/smith-and-web/kindling")}
          class="flex items-center gap-2 w-full px-3 py-2 text-press-ui text-press-muted hover:text-press-text hover:bg-press-sunken rounded-lg transition-colors"
        >
          <ExternalLink class="w-4 h-4 shrink-0" />
          GitHub Repository
        </button>
        <button
          onclick={() => openLink("https://github.com/smith-and-web/kindling/issues/new")}
          class="flex items-center gap-2 w-full px-3 py-2 text-press-ui text-press-muted hover:text-press-text hover:bg-press-sunken rounded-lg transition-colors"
        >
          <ExternalLink class="w-4 h-4 shrink-0" />
          Report an Issue
        </button>
        <button
          onclick={() => openLink("https://github.com/smith-and-web/kindling/releases")}
          class="flex items-center gap-2 w-full px-3 py-2 text-press-ui text-press-muted hover:text-press-text hover:bg-press-sunken rounded-lg transition-colors"
        >
          <ExternalLink class="w-4 h-4 shrink-0" />
          Release Notes
        </button>
      </div>
    </div>

    <div
      class="px-5 py-3 border-t border-press-border text-center text-press-eyebrow text-press-muted shrink-0"
    >
      &copy; 2026 Josh Smith
    </div>
  </div>
</div>
