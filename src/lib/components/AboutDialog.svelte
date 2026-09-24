<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { ChevronRight, ExternalLink, Send } from "lucide-svelte";
  import DialogHeader from "./DialogHeader.svelte";
  import { onMount } from "svelte";
  import { modalFocus } from "../utils/modalFocus";

  let { onClose, onSendFeedback }: { onClose: () => void; onSendFeedback: () => void } = $props();

  let version = $state("…");

  onMount(async () => {
    try {
      version = await getVersion();
    } catch {
      version = "unknown";
    }
  });

  let linkError = $state<string | null>(null);

  async function openLink(url: string) {
    linkError = null;
    try {
      await openUrl(url);
    } catch (error) {
      linkError = `Couldn’t open the link: ${error}`;
    }
  }
</script>

<div
  class="dialog-scrim"
  use:modalFocus={{ onEscape: onClose }}
  role="dialog"
  aria-modal="true"
  aria-labelledby="about-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-narrow dialog-shell" data-testid="about-dialog">
    <DialogHeader
      title="About kindling"
      titleId="about-title"
      {onClose}
      closeLabel="Close"
      closeTestId="about-close"
    />

    <div class="ka-dialog-body about">
      <div class="about-identity">
        <img class="on-light" src="/brand/kindling-mark.svg" alt="" width="88" height="50" />
        <img
          class="on-dark"
          src="/brand/kindling-mark-reversed.svg"
          alt=""
          width="88"
          height="50"
        />
        <p class="about-tagline">Spark your draft — bridge the gap between outline and prose.</p>
      </div>

      <dl class="ka-facts">
        <div>
          <dt>Version</dt>
          <dd>{version}</dd>
        </div>
      </dl>

      <ul class="about-links">
        <li>
          <button type="button" onclick={onSendFeedback} class="about-link">
            <Send class="w-5 h-5" aria-hidden="true" />
            <span>Send feedback</span>
            <ChevronRight class="w-5 h-5" aria-hidden="true" />
          </button>
        </li>
        <li>
          <button
            type="button"
            onclick={() => openLink("https://github.com/smith-and-web/kindling")}
            class="about-link"
          >
            <ExternalLink class="w-5 h-5" aria-hidden="true" />
            <span>GitHub repository</span>
          </button>
        </li>
        <li>
          <button
            type="button"
            onclick={() => openLink("https://github.com/smith-and-web/kindling/issues/new")}
            class="about-link"
          >
            <ExternalLink class="w-5 h-5" aria-hidden="true" />
            <span>Report an issue</span>
          </button>
        </li>
        <li>
          <button
            type="button"
            onclick={() => openLink("https://github.com/smith-and-web/kindling/releases")}
            class="about-link"
          >
            <ExternalLink class="w-5 h-5" aria-hidden="true" />
            <span>Release notes</span>
          </button>
        </li>
      </ul>
      {#if linkError}
        <p class="ka-error" role="alert">{linkError}</p>
      {/if}

      <p class="ka-help">&copy; 2026 Josh Smith</p>
    </div>
  </div>
</div>

<style>
  .about {
    display: grid;
    gap: var(--space-s);
  }
  .about-identity {
    display: grid;
    justify-items: start;
    gap: var(--space-xs);
  }
  .about-identity img {
    display: block;
    width: 88px;
    height: auto;
  }
  .on-dark {
    display: none !important;
  }
  :global([data-theme="dark"]) .about-identity .on-light {
    display: none !important;
  }
  :global([data-theme="dark"]) .about-identity .on-dark {
    display: block !important;
  }
  .about-tagline {
    margin: 0;
    max-width: var(--measure);
    font: italic var(--text-body) / var(--leading-relaxed) var(--font-body);
    color: var(--color-text);
  }
  .about-links {
    display: grid;
    margin: 0;
    padding: 0;
    list-style: none;
    border-top: var(--border-hair);
  }
  .about-links li {
    border-bottom: var(--border-hair);
  }
  .about-link {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    width: 100%;
    min-height: var(--control-target);
    padding: var(--space-2xs) var(--space-3xs);
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--color-text);
    font: var(--text-ui) / 1.5 var(--font-ui);
    text-align: left;
    cursor: pointer;
  }
  .about-link span {
    flex: 1;
  }
  .about-link :global(svg) {
    color: var(--color-text-muted);
  }
  @media (hover: hover) {
    .about-link:hover {
      background: var(--color-surface-sunken);
    }
  }
</style>
