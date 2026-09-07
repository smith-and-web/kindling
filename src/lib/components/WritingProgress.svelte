<script lang="ts">
  import { untrack } from "svelte";
  import { currentProject } from "../stores/project.svelte";
  import { writing } from "../stores/writing.svelte";
  import { proseSaves } from "../utils/proseSaves";

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
        resetError = `Save your pending prose before resetting: ${String(error)}`;
    } finally {
      resetting = false;
    }
  }
</script>

{#if writing.value && writing.value.project_id === currentProject.value?.id}
  {@const stats = writing.value}
  <div class="mt-3 space-y-1 text-press-eyebrow text-press-muted" data-testid="writing-progress">
    <p>{stats.project_words.toLocaleString()} project words</p>
    <p title="Net words added through saved edits today. Deletions reduce this total.">
      Today: {stats.today_words.toLocaleString()}{#if stats.daily_goal > 0}
        / {stats.daily_goal.toLocaleString()} words{:else}
        words · goal off{/if}
    </p>
    {#if stats.daily_goal > 0}
      <progress
        class="w-full"
        aria-label="Daily writing goal"
        max={stats.daily_goal}
        value={Math.max(0, stats.today_words)}
      ></progress>
    {/if}
    <div class="flex items-center justify-between gap-2">
      <span title="Net words saved in this project since opening the app or resetting."
        >Session: {stats.session_words.toLocaleString()} words</span
      >
      <button
        class="text-press-muted hover:text-press-text"
        onclick={reset}
        disabled={resetting}
        aria-label="Reset writing session">Reset</button
      >
    </div>
    <p>{stats.streak} day{stats.streak === 1 ? "" : "s"} writing streak</p>
  </div>
{/if}
{#if resetError}
  <p role="alert" class="mt-2 text-press-eyebrow text-press-error">{resetError}</p>
{/if}
{#if writing.error}
  <p role="alert" class="mt-2 text-press-eyebrow text-press-error">{writing.error}</p>
{/if}

<style>
  progress {
    appearance: none;
    height: var(--space-3xs);
    background: var(--color-surface-sunken);
    border: none;
  }
  progress::-webkit-progress-bar {
    background: var(--color-surface-sunken);
  }
  progress::-webkit-progress-value {
    background: var(--color-accent);
  }
  progress::-moz-progress-bar {
    background: var(--color-accent);
  }
</style>
