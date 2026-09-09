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
  <div class="mb-6" aria-busy={loading}>
    <div class="flex flex-wrap items-center gap-x-6 gap-y-2" role="group" aria-label="Scene tools">
      {#if previous}
        <button
          type="button"
          onclick={toggle}
          aria-expanded={!collapsed}
          aria-controls="previously-content"
          class="flex items-center gap-2 py-1 text-press-ui font-press-ui text-press-muted hover:text-press-text"
        >
          <ChevronDown
            class={collapsed ? "w-4 h-4 -rotate-90" : "w-4 h-4"}
            strokeWidth={1}
            aria-hidden="true"
          />
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
        class="mt-4 space-y-3"
      >
        <h2 class="font-heading text-press-h3 text-press-text break-words">{title}</h2>
        {#if synopsis}
          <p
            class="font-prose text-press-body text-press-text max-w-press-measure whitespace-pre-wrap break-words"
          >
            {synopsis}
          </p>
        {/if}
        {#if previous.excerpt}
          <blockquote
            class="font-prose text-press-body text-press-text max-w-press-measure break-words"
          >
            {previous.excerpt}
          </blockquote>
        {/if}
      </section>
    {:else if error}
      <p role="status" class="mt-3 text-press-ui text-press-muted">
        Could not load previous scene context.
        <button type="button" onclick={() => retry++} class="underline text-press-text"
          >Retry</button
        >
      </p>
    {/if}
  </div>
{/if}
