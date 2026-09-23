<script lang="ts">
  import { tick, type Snippet } from "svelte";
  import { ChevronDown } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { synopsisSaves } from "../stores/synopsisSaves.svelte";
  import { loadPreviousScene } from "../utils/previousScene";

  let {
    actions,
    refreshVersion = 0,
    loading = $bindable(true),
  }: { actions?: Snippet; refreshVersion?: number; loading?: boolean } = $props();

  const storageKey = "kindling:previouslyCollapsed";
  let collapsed = $state(localStorage.getItem(storageKey) === "true");
  let previous = $state<Awaited<ReturnType<typeof loadPreviousScene>>>(null);
  let error = $state(false);
  let retry = $state(0);
  const synopsis = $derived.by(() => {
    if (!previous) return null;
    const draft = synopsisSaves.getState(currentProject.value?.id, previous.scene.id).draft;
    const scene = currentProject.scenes.find((item) => item.id === previous?.scene.id);
    return draft ? draft.synopsis : (scene ?? previous.scene).synopsis;
  });
  const title = $derived(
    currentProject.scenes.find((item) => item.id === previous?.scene.id)?.title ??
      previous?.scene.title
  );
  // Observe navigation and outline structure, without refetching on each prose edit.
  const target = $derived(
    JSON.stringify([
      currentProject.value?.id,
      currentProject.currentScene?.id,
      currentProject.currentScene?.chapter_id,
      currentProject.chapters.map((chapter) => [chapter.id, chapter.position, chapter.archived]),
      currentProject.scenes.map((scene) => [scene.id, scene.position, scene.archived]),
      retry,
      refreshVersion,
    ])
  );
  $effect(() => {
    void target;
    let cancelled = false;
    previous = null;
    error = false;
    loading = true;
    // Let ScenePanel/BeatView submit their navigation saves first.
    void tick().then(async () => {
      if (cancelled) return;
      const project = currentProject.value;
      const scene = currentProject.currentScene;
      if (!project || !scene) {
        loading = false;
        return;
      }
      try {
        const result = await loadPreviousScene(project.id, scene, currentProject.chapters);
        if (!cancelled) previous = result;
      } catch {
        if (!cancelled) error = true;
      } finally {
        if (!cancelled) loading = false;
      }
    });
    return () => {
      cancelled = true;
    };
  });

  function toggle() {
    collapsed = !collapsed;
    localStorage.setItem(storageKey, String(collapsed));
  }
</script>

{#if previous || actions || error}
  <div class="previously" aria-busy={loading}>
    <div class="previously-tools" role="group" aria-label="Scene tools">
      {#if previous}
        <button
          type="button"
          onclick={toggle}
          aria-expanded={!collapsed}
          aria-controls="previously-content"
          class="ka-button ka-button--ghost previously-toggle"
        >
          <ChevronDown class={collapsed ? "w-5 h-5 -rotate-90" : "w-5 h-5"} aria-hidden="true" />
          Previously
        </button>
      {/if}
      {@render actions?.()}
    </div>
    {#if previous && !collapsed}
      <section
        id="previously-content"
        aria-label="Previously"
        data-testid="previously"
        class="previously-body"
      >
        <h2>{title}</h2>
        {#if synopsis}
          <p>
            {synopsis}
          </p>
        {/if}
        {#if previous.excerpt}
          <blockquote>
            {previous.excerpt}
          </blockquote>
        {/if}
      </section>
    {:else if error}
      <p role="status" class="ka-help previously-error">
        Could not load previous scene context.
        <button type="button" onclick={() => retry++} class="ka-button ka-button--ghost"
          >Retry</button
        >
      </p>
    {/if}
  </div>
{/if}

<style>
  .previously {
    margin-bottom: var(--space-m);
  }
  .previously-toggle {
    margin-left: calc(-1 * var(--space-s));
    color: var(--color-text-muted);
  }
  .previously-body {
    display: grid;
    gap: var(--space-xs);
    margin-top: var(--space-2xs);
    padding: var(--space-s) 0 var(--space-s) var(--space-s);
    border-left: 2px solid var(--color-border);
  }
  .previously-body h2 {
    margin: 0;
    font: 550 var(--text-h3) / var(--leading-tight) var(--font-display);
    letter-spacing: var(--tracking-tight);
    color: var(--color-text);
    overflow-wrap: anywhere;
  }
  .previously-body :where(p, blockquote) {
    margin: 0;
    max-width: var(--measure);
    font: var(--text-body) / var(--leading-relaxed) var(--font-body);
    color: var(--color-text);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .previously-body blockquote {
    font-style: italic;
  }
  .previously-error {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
  }
</style>
