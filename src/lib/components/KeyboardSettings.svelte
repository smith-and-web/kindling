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

<div class="settings-pane">
  <div class="keys-intro">
    <p class="settings-lede">
      Choose a shortcut, then press a new key combination. Changes apply immediately to all
      projects. Escape cancels recording; Tab moves to the next control.
    </p>
    <p class="ka-help">
      Standard text editing and system controls (such as copy, paste, undo, Tab, and Escape) keep
      their usual keys.
    </p>
  </div>
  {#if shortcuts.error}
    <div class="ka-notice ka-notice--error" role="alert">
      <p>{shortcuts.error}</p>
      <button
        type="button"
        class="ka-button ka-button--secondary keys-retry"
        onclick={async () => {
          await shortcuts.load();
          await shortcuts.suspend(true);
        }}>Retry loading shortcuts</button
      >
    </div>
  {/if}
  {#if error}<p role="alert" class="ka-error">{error}</p>{/if}
  <p role="status" class="ka-help keys-message">{message}</p>
  <div class="keys-toolbar">
    <div class="ka-field od-field keys-filter">
      <label for="shortcut-filter">Filter commands</label>
      <input id="shortcut-filter" type="search" bind:value={filter} />
    </div>
    <button
      type="button"
      disabled={busy || !shortcuts.suspended}
      onclick={() => save({ ...defaultBindings })}
      class="ka-button ka-button--secondary">Reset all to defaults</button
    >
  </div>
  {#if (!shortcuts.ready || !shortcuts.suspended) && !shortcuts.error}<p
      role="status"
      class="ka-help"
    >
      Loading shortcuts…
    </p>{/if}
  <ul class="keys-list">
    {#each shown as def (def.id)}
      <li class="keys-row">
        <div class="keys-name">
          <p>{def.label}</p>
          <small>{def.category}</small>
        </div>
        <button
          type="button"
          aria-label={`Shortcut for ${def.label}`}
          aria-describedby={`shortcut-binding-${def.id}`}
          aria-pressed={recording === def.id}
          disabled={busy || !shortcuts.ready || !shortcuts.suspended}
          onclick={(event) => {
            // WebKit on macOS does not focus buttons when they are clicked.
            event.currentTarget.focus();
            recording = def.id;
            error = "";
            message = "";
          }}
          onkeydown={(event) => capture(event, def.id)}
          onblur={() => {
            if (recording === def.id) recording = null;
          }}
          class="ka-button ka-button--secondary keys-binding"
          class:is-recording={recording === def.id}
          class:is-unassigned={!shortcuts.label(def.id) && recording !== def.id}
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
          class="ka-button ka-button--ghost">Clear</button
        >
      </li>
    {/each}
  </ul>
  {#if shown.length === 0}<p class="ka-help">No matching commands.</p>{/if}
</div>

<style>
  .keys-intro {
    display: grid;
    gap: var(--space-2xs);
  }
  .keys-intro p {
    margin: 0;
  }
  .keys-message:empty {
    display: none;
  }
  .keys-retry {
    justify-self: start;
  }
  .keys-toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: var(--space-s);
  }
  .keys-filter {
    flex: 1 1 16rem;
    width: auto;
  }
  .keys-list {
    margin: 0;
    padding: 0;
    list-style: none;
    border-top: var(--border-hair);
  }
  .keys-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2xs) var(--space-xs);
    padding: var(--space-2xs) 0;
    border-bottom: var(--border-hair);
  }
  .keys-name {
    flex: 1 1 12rem;
    min-width: 0;
  }
  .keys-name p {
    margin: 0;
    font: var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .keys-name small {
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .keys-binding {
    min-width: 8rem;
    font-family: var(--font-ui);
    font-size: var(--text-small);
    font-variant-numeric: tabular-nums;
  }
  .keys-binding.is-unassigned {
    font-family: var(--font-ui);
    color: var(--color-text-muted);
  }
  .keys-binding.is-recording {
    box-shadow: inset 0 0 0 2px var(--color-accent-text);
    color: var(--color-accent-text);
  }
</style>
