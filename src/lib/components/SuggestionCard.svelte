<script lang="ts">
  import { Link2, X } from "lucide-svelte";
  import type { ReferenceSuggestion } from "../types";
  import Tooltip from "./Tooltip.svelte";

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

  const confidenceColor = $derived.by(() => {
    if (suggestion.confidence >= 0.9) return "text-press-success";
    if (suggestion.confidence >= 0.5) return "text-press-warning";
    return "text-press-muted";
  });

  const typeLabel = $derived.by(() => {
    const t = suggestion.reference_type;
    if (t === "character") return "Character";
    if (t === "location") return "Location";
    return t.charAt(0).toUpperCase() + t.slice(1);
  });
</script>

<div
  class="flex items-start gap-2 p-2 bg-press-sunken rounded-lg border border-press-border hover:border-press-accent transition-colors"
>
  <div class="flex-1 min-w-0">
    <div class="flex items-center gap-1.5">
      <span class="text-press-ui font-medium text-press-text truncate">
        {suggestion.reference_name}
      </span>
      <span class="text-press-eyebrow px-1.5 py-0.5 rounded-full bg-press-sunken text-press-muted">
        {typeLabel}
      </span>
      <span class="text-press-eyebrow {confidenceColor}">
        {confidenceLabel}
      </span>
    </div>
    <p class="text-press-eyebrow text-press-muted mt-0.5 truncate">
      &ldquo;{suggestion.match_text}&rdquo;
    </p>
  </div>
  <div class="flex items-center gap-1 shrink-0">
    <Tooltip text="Link to scene" position="left">
      <button
        onclick={() => onLink(suggestion)}
        class="p-1 rounded hover:bg-press-accent-wash text-press-accent-text transition-colors cursor-pointer"
        aria-label="Link {suggestion.reference_name} to scene"
      >
        <Link2 class="w-3.5 h-3.5" />
      </button>
    </Tooltip>
    <Tooltip text="Dismiss" position="left">
      <button
        onclick={() => onDismiss(suggestion)}
        class="p-1 rounded hover:bg-press-error-wash text-press-muted hover:text-press-error transition-colors cursor-pointer"
        aria-label="Dismiss suggestion for {suggestion.reference_name}"
      >
        <X class="w-3.5 h-3.5" />
      </button>
    </Tooltip>
  </div>
</div>
