<script lang="ts">
  import { ui } from "../stores/ui.svelte";

  let { onStartTour }: { onStartTour?: () => void } = $props();

  // Each tip shows once; this brings them back without switching tips off and on.
  let tipsReset = $state(false);
  function showTipsAgain() {
    ui.resetGuidanceTooltips();
    tipsReset = true;
  }
</script>

<div class="settings-pane">
  <p class="settings-lede">
    Choose how kindling looks and how much guidance it shows. Changes apply immediately to all
    projects.
  </p>
  <div class="ka-group">
    <fieldset class="settings-fieldset">
      <legend class="ka-group-title">Appearance</legend>
      <div class="ka-segments">
        <div class="ka-segment-track" role="radiogroup" aria-label="Theme">
          {#each [{ value: "dark", label: "Dark" }, { value: "light", label: "Light" }, { value: "system", label: "System" }] as opt}
            <label class="ka-segment" class:ka-selected={ui.theme === opt.value}>
              <input
                type="radio"
                name="theme"
                data-testid="theme-option-{opt.value}"
                value={opt.value}
                checked={ui.theme === opt.value}
                onchange={() => ui.setTheme(opt.value as "dark" | "light" | "system")}
              />
              <span>{opt.label}</span>
            </label>
          {/each}
        </div>
      </div>
      <p class="ka-help">“System” follows your operating system’s appearance setting.</p>
    </fieldset>
  </div>

  <div class="ka-group">
    <fieldset class="settings-fieldset">
      <legend class="ka-group-title">Guidance</legend>
      <div class="ka-check">
        <input
          id="guidance-enabled"
          type="checkbox"
          checked={ui.guidanceEnabled}
          onchange={(e) => ui.setGuidanceEnabled((e.target as HTMLInputElement).checked)}
          aria-describedby="guidance-enabled-help"
        />
        <label for="guidance-enabled">Show guidance tips</label>
      </div>
      <p id="guidance-enabled-help" class="ka-help">
        A tip appears the first time you visit the sidebar, scene panel and references. Each tip
        shows once.
      </p>
      <div class="guidance-actions">
        <button
          type="button"
          class="ka-button ka-button--secondary"
          onclick={showTipsAgain}
          disabled={!ui.guidanceEnabled}
          aria-describedby="guidance-reset-status"
        >
          Show tips again
        </button>
        {#if onStartTour}
          <button type="button" class="ka-button ka-button--secondary" onclick={onStartTour}>
            Take the tour
          </button>
        {/if}
      </div>
      <p id="guidance-reset-status" class="ka-help" role="status">
        {#if !ui.guidanceEnabled}
          Turn on guidance tips to show them again.
        {:else if tipsReset}
          Tips will appear again as you visit each area.
        {/if}
      </p>
    </fieldset>
  </div>
</div>

<style>
  .guidance-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2xs);
  }
  #guidance-reset-status:empty {
    display: none;
  }
</style>
