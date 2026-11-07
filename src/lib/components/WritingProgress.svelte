<script lang="ts">
  import { untrack } from "svelte";
  import { currentProject } from "../stores/project.svelte";
  import { writing } from "../stores/writing.svelte";
  import { proseSaves } from "../utils/proseSaves";
  import { t, tn } from "../i18n";
  import { formatNumber } from "../i18n/format";

  let { prepareReset }: { prepareReset?: () => Promise<void> } = $props();
  let resetting = $state(false);
  let resetError = $state<string | null>(null);
  const projectId = $derived(currentProject.value?.id ?? null);
  $effect(() => {
    const id = projectId;
    untrack(() => {
      writing.open(id);
      resetError = null;
    });
    // Refresh after midnight and structural edits, even if no prose was saved.
    const timer = setInterval(() => void writing.refresh(id), 30000);
    return () => {
      clearInterval(timer);
      writing.open(null);
    };
  });

  async function reset() {
    const projectId = currentProject.value?.id;
    if (!projectId) return;
    resetting = true;
    resetError = null;
    try {
      await prepareReset?.();
      await proseSaves.flush(projectId);
      await writing.reset(projectId);
    } catch (error) {
      if (currentProject.value?.id === projectId)
        resetError = t("editor.writingProgress.resetError", { error: String(error) });
    } finally {
      resetting = false;
    }
  }
</script>

{#if writing.value && writing.value.project_id === currentProject.value?.id}
  {@const stats = writing.value}
  <div class="progress" data-testid="writing-progress">
    <p>
      {tn("editor.writingProgress.projectWords", stats.project_words, {
        count: formatNumber(stats.project_words),
      })}
    </p>
    <div class="ka-progress progress-goal">
      <p title={t("editor.writingProgress.todayTooltip")}>
        {#if stats.daily_goal > 0}
          {tn("editor.writingProgress.todayWithGoal", stats.daily_goal, {
            today: formatNumber(stats.today_words),
            goal: formatNumber(stats.daily_goal),
          })}
        {:else}
          {tn("editor.writingProgress.todayWithoutGoal", stats.today_words, {
            today: formatNumber(stats.today_words),
          })}
        {/if}
      </p>
      {#if stats.daily_goal > 0}
        <progress
          aria-label={t("editor.writingProgress.dailyGoal")}
          max={stats.daily_goal}
          value={Math.max(0, stats.today_words)}
        ></progress>
      {/if}
    </div>
    <div class="ka-between progress-session">
      <span title={t("editor.writingProgress.sessionTooltip")}
        >{tn("editor.writingProgress.sessionWords", stats.session_words, {
          count: formatNumber(stats.session_words),
        })}</span
      >
      <button
        type="button"
        class="ka-button ka-button--ghost"
        onclick={reset}
        disabled={resetting}
        aria-busy={resetting || undefined}
        aria-label={t("editor.writingProgress.resetSession")}
        >{t("editor.writingProgress.reset")}</button
      >
    </div>
    <p>
      {tn("editor.writingProgress.streak", stats.streak)}
    </p>
  </div>
{/if}
{#if resetError}
  <p role="alert" class="progress-error">{resetError}</p>
{/if}

<style>
  .progress {
    display: grid;
    gap: var(--space-2xs);
    padding: var(--space-3xs) 0 var(--space-s) var(--space-xs);
    border-bottom: var(--border-hair);
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .progress p {
    margin: 0;
  }
  .progress-goal {
    display: grid;
    gap: var(--space-3xs);
  }
  .progress-session .ka-button {
    min-height: var(--control-target);
    margin-block: calc(-1 * var(--space-2xs));
    padding: var(--space-2xs) var(--space-xs);
    font-size: var(--text-small);
  }
  .progress-error {
    margin: var(--space-2xs) 0 0 var(--space-xs);
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-error);
  }
</style>
