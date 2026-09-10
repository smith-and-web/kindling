<script lang="ts">
  import { onMount } from "svelte";
  import definitions from "../shortcutDefinitions.json";
  import { shortcuts } from "../stores/shortcuts.svelte";
  import { bindingProblem, defaultBindings, shortcutFromEvent } from "../utils/keyboardShortcuts";
  let { busy = $bindable(false) }: { busy?: boolean } = $props();
  let recording = $state<string | null>(null);
  let message = $state("");
  let error = $state("");
  let filter = $state("");
  const shown = $derived(
    definitions.filter((def) =>
      `${def.label} ${def.category}`.toLowerCase().includes(filter.toLowerCase())
    )
  );
  $effect(() => {
    shortcuts.recording = recording !== null;
    return () => {
      shortcuts.recording = false;
    };
  });
  onMount(() => {
    void shortcuts.load();
  });

  async function save(bindings: Record<string, string>) {
    busy = true;
    error = "";
    message = "";
    try {
      await shortcuts.save(bindings);
      recording = null;
      message = "Keyboard shortcuts saved.";
    } catch (e) {
      error = `Could not save keyboard shortcuts: ${String(e)}`;
    } finally {
      busy = false;
    }
  }

  function capture(event: KeyboardEvent, id: string) {
    if (recording !== id) return;
    event.stopPropagation();
    if (event.key === "Tab") {
      recording = null;
      return;
    }
    event.preventDefault();
    if (event.key === "Escape") {
      recording = null;
      error = "";
      return;
    }
    if (busy || event.repeat || ["Meta", "Control", "Alt", "Shift"].includes(event.key)) return;
    const binding = shortcutFromEvent(event);
    if (!binding) {
      error =
        "Use Command on macOS or Ctrl on Windows/Linux, with a letter, number, punctuation key, or F1–F24.";
      return;
    }
    const problem = bindingProblem(id, binding, shortcuts.bindings);
    if (problem) {
      error = problem;
      return;
    }
    void save({ ...shortcuts.bindings, [id]: binding });
  }
</script>

<div class="space-y-4">
  <p class="text-press-ui">
    Choose a shortcut, then press a new key combination. Changes apply immediately to all projects.
    Escape cancels recording; Tab moves to the next control.
  </p>
  <p class="text-press-ui text-press-muted">
    Standard text editing and system controls (such as copy, paste, undo, Tab, and Escape) keep
    their usual keys.
  </p>
  {#if shortcuts.error}
    <p role="alert" class="text-press-ui text-press-error">{shortcuts.error}</p>
    <button
      type="button"
      onclick={async () => {
        await shortcuts.load();
        await shortcuts.suspend(true);
      }}>Retry loading shortcuts</button
    >
  {/if}
  {#if error}<p role="alert" class="text-press-ui text-press-error">{error}</p>{/if}
  <p role="status" class="text-press-ui text-press-muted">{message}</p>
  <div class="flex flex-wrap gap-3 items-end">
    <label class="flex-1 text-press-ui"
      >Filter commands
      <input
        type="search"
        bind:value={filter}
        class="block w-full mt-1 px-3 py-2 border border-press-border hover:border-press-accent focus:border-press-accent rounded"
      />
    </label>
    <button
      type="button"
      disabled={busy || !shortcuts.suspended}
      onclick={() => save({ ...defaultBindings })}
      class="px-3 py-2 border border-press-border rounded text-press-ui"
      >Reset all to defaults</button
    >
  </div>
  {#if (!shortcuts.ready || !shortcuts.suspended) && !shortcuts.error}<p role="status">
      Loading shortcuts…
    </p>{/if}
  <ul class="divide-y divide-press-border">
    {#each shown as def (def.id)}
      <li class="flex flex-wrap items-center gap-3 py-3">
        <div class="flex-1 min-w-0">
          <p class="text-press-ui">{def.label}</p>
          <p class="text-press-eyebrow text-press-muted">{def.category}</p>
        </div>
        <button
          type="button"
          aria-label={`Shortcut for ${def.label}`}
          aria-describedby={`shortcut-binding-${def.id}`}
          aria-pressed={recording === def.id}
          disabled={busy || !shortcuts.ready || !shortcuts.suspended}
          onclick={() => {
            recording = def.id;
            error = "";
            message = "";
          }}
          onkeydown={(event) => capture(event, def.id)}
          onblur={() => {
            if (recording === def.id) recording = null;
          }}
          class="min-w-28 px-3 py-2 border border-press-border rounded text-press-ui"
        >
          <span id={`shortcut-binding-${def.id}`}
            >{recording === def.id ? "Press keys…" : shortcuts.label(def.id) || "Unassigned"}</span
          >
        </button>
        <button
          type="button"
          aria-label={`Clear shortcut for ${def.label}`}
          disabled={busy || !shortcuts.ready || !shortcuts.suspended || !shortcuts.bindings[def.id]}
          onclick={() => save({ ...shortcuts.bindings, [def.id]: "" })}
          class="px-2 py-2 text-press-ui text-press-muted">Clear</button
        >
      </li>
    {/each}
  </ul>
  {#if shown.length === 0}<p class="text-press-ui">No matching commands.</p>{/if}
</div>
