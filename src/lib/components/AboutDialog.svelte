<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { X, ExternalLink, Flame } from "lucide-svelte";
  import { onMount } from "svelte";

  let { onClose }: { onClose: () => void } = $props();

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
  class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
  role="dialog"
  aria-modal="true"
  aria-labelledby="about-title"
  tabindex="-1"
>
  <div
    class="bg-bg-panel rounded-xl shadow-xl max-w-sm w-full flex flex-col"
    data-testid="about-dialog"
  >
    <div class="flex items-center justify-between p-5 border-b border-bg-card shrink-0">
      <h2 id="about-title" class="text-lg font-heading font-semibold text-text-primary">
        About Kindling
      </h2>
      <button
        onclick={onClose}
        class="p-1 rounded hover:bg-bg-card text-text-secondary transition-colors"
        aria-label="Close"
      >
        <X class="w-5 h-5" />
      </button>
    </div>

    <div class="p-5 flex flex-col items-center text-center gap-4">
      <div class="w-14 h-14 rounded-2xl bg-accent/15 flex items-center justify-center">
        <Flame class="w-8 h-8 text-accent" />
      </div>

      <div>
        <h3 class="text-lg font-heading font-semibold text-text-primary">Kindling</h3>
        <p class="text-sm text-text-secondary mt-0.5">Version {version}</p>
      </div>

      <p class="text-sm text-text-secondary leading-relaxed">
        Spark your draft &mdash; Bridge the gap between outline and prose.
      </p>

      <div class="w-full border-t border-bg-card pt-4 flex flex-col gap-2">
        <button
          onclick={() => openLink("https://github.com/smith-and-web/kindling")}
          class="flex items-center gap-2 w-full px-3 py-2 text-sm text-text-secondary hover:text-text-primary hover:bg-bg-card rounded-lg transition-colors"
        >
          <ExternalLink class="w-4 h-4 shrink-0" />
          GitHub Repository
        </button>
        <button
          onclick={() => openLink("https://github.com/smith-and-web/kindling/issues/new")}
          class="flex items-center gap-2 w-full px-3 py-2 text-sm text-text-secondary hover:text-text-primary hover:bg-bg-card rounded-lg transition-colors"
        >
          <ExternalLink class="w-4 h-4 shrink-0" />
          Report an Issue
        </button>
        <button
          onclick={() => openLink("https://github.com/smith-and-web/kindling/releases")}
          class="flex items-center gap-2 w-full px-3 py-2 text-sm text-text-secondary hover:text-text-primary hover:bg-bg-card rounded-lg transition-colors"
        >
          <ExternalLink class="w-4 h-4 shrink-0" />
          Release Notes
        </button>
      </div>
    </div>

    <div class="px-5 py-3 border-t border-bg-card text-center text-xs text-text-secondary shrink-0">
      &copy; 2026 Josh Smith
    </div>
  </div>
</div>
