<script lang="ts">
  import { Link2, X } from "lucide-svelte";
  import type { ReferenceSuggestion } from "../types";

  let {
    suggestion,
    onLink,
    onDismiss,
  }: {
    suggestion: ReferenceSuggestion;
    onLink: (suggestion: ReferenceSuggestion) => void;
    onDismiss: (suggestion: ReferenceSuggestion) => void;
  } = $props();

  const confidenceLabel = $derived.by(() => {
    if (suggestion.confidence >= 0.9) return "High";
    if (suggestion.confidence >= 0.5) return "Medium";
    return "Low";
  });

  // The badge text carries the meaning; tone only reinforces it.
  const confidenceTone = $derived.by(() => {
    if (suggestion.confidence >= 0.9) return "ka-badge--success";
    if (suggestion.confidence >= 0.5) return "ka-badge--warning";
    return "";
  });

  const typeLabel = $derived.by(() => {
    const t = suggestion.reference_type;
    if (t === "character") return "Character";
    if (t === "location") return "Location";
    return t.charAt(0).toUpperCase() + t.slice(1);
  });
</script>

<div class="suggestion">
  <div class="suggestion-main">
    <div class="suggestion-title">
      <span class="suggestion-name">{suggestion.reference_name}</span>
      <span class="ka-badge {confidenceTone}">{confidenceLabel}</span>
    </div>
    <small class="suggestion-match" title={suggestion.match_text}>
      {typeLabel} · found “{suggestion.match_text}”
    </small>
  </div>
  <button
    type="button"
    onclick={() => onLink(suggestion)}
    class="ka-button ka-button--secondary suggestion-link"
    aria-label="Link {suggestion.reference_name} to scene"
    title="Link to scene"
  >
    <Link2 class="w-4 h-4" aria-hidden="true" />
    Link
  </button>
  <button
    type="button"
    onclick={() => onDismiss(suggestion)}
    class="ka-button ka-button--ghost ka-icon-button suggestion-dismiss"
    aria-label="Dismiss suggestion for {suggestion.reference_name}"
    title="Dismiss"
  >
    <X class="w-5 h-5" aria-hidden="true" />
  </button>
</div>

<style>
  .suggestion {
    display: flex;
    align-items: center;
    gap: var(--space-3xs);
    min-height: 56px;
  }
  .suggestion-main {
    display: grid;
    flex: 1;
    min-width: 0;
    padding: var(--space-2xs);
  }
  .suggestion-title {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    min-width: 0;
  }
  .suggestion-name {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: 500 var(--text-ui) / 1.4 var(--font-ui);
    color: var(--color-text);
  }
  .suggestion-title .ka-badge {
    min-height: 20px;
    padding: 2px 6px;
    font-size: var(--text-eyebrow);
  }
  .suggestion-match {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .suggestion-link {
    flex: none;
    padding: var(--space-2xs) var(--space-xs);
    font-size: var(--text-small);
  }
  .suggestion-dismiss {
    flex: none;
    color: var(--color-text-muted);
  }
  @media (hover: hover) {
    .suggestion-dismiss:hover {
      color: var(--color-error);
    }
  }
  /* Keyed to the references panel's own width, which can be narrow in any window. */
  @container refs (max-width: 360px) {
    .suggestion {
      flex-wrap: wrap;
    }
    .suggestion-main {
      flex-basis: 100%;
    }
    .suggestion-link {
      margin-left: auto;
    }
  }
</style>
