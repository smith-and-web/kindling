<script lang="ts">
  import { tagColor } from "../utils/tagColor";
  import { onDestroy, untrack } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Loader2, Pencil, Plus, Trash2, X, Check } from "lucide-svelte";
  import pressTokens from "../../styles/press/tokens.json";
  import type { Tag } from "../types";

  let {
    projectId,
    onState,
    onChange,
  }: {
    projectId: string;
    onState?: (state: { dirty: boolean; busy: boolean }) => void;
    onChange?: () => void;
  } = $props();

  let tags = $state<Tag[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let editingTag = $state<Partial<Tag> | null>(null);

  /** Land keyboard users on the editor (and scroll it into view) when it opens. */
  function focusOnMount(node: HTMLElement) {
    node.focus();
  }
  let editMode = $state<"create" | "edit">("create");
  let saving = $state(false);
  $effect(() => {
    const state = { dirty: editingTag !== null, busy: saving };
    untrack(() => onState?.(state));
  });
  onDestroy(() => onState?.({ dirty: false, busy: false }));

  const PRESET_COLOR_TOKENS: Array<[string, keyof typeof pressTokens.light]> = [
    ["Red", "--tag-red"],
    ["Orange", "--tag-orange"],
    ["Yellow", "--tag-yellow"],
    ["Green", "--tag-green"],
    ["Teal", "--tag-teal"],
    ["Blue", "--tag-blue"],
    ["Purple", "--tag-purple"],
    ["Pink", "--tag-pink"],
    ["Slate", "--tag-slate"],
    ["Cyan", "--tag-cyan"],
  ];
  const PRESET_COLORS = PRESET_COLOR_TOKENS.map(([name, token]) => ({
    name,
    color: pressTokens.light[token],
  }));

  async function loadTags() {
    loading = true;
    error = null;
    try {
      tags = await invoke("get_tags", { projectId });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (projectId) loadTags();
  });

  function getRootTags(): Tag[] {
    return tags.filter((t) => !t.parent_id).sort((a, b) => a.position - b.position);
  }

  function getChildTags(parentId: string): Tag[] {
    return tags.filter((t) => t.parent_id === parentId).sort((a, b) => a.position - b.position);
  }

  function openCreateForm(parentId: string | null = null) {
    editMode = "create";
    editingTag = {
      name: "",
      color: null,
      parent_id: parentId,
    };
  }

  function openEditForm(tag: Tag) {
    editMode = "edit";
    editingTag = { ...tag };
  }

  function cancelEdit() {
    editingTag = null;
  }

  async function saveTag() {
    if (!editingTag?.name?.trim()) return;

    saving = true;
    try {
      if (editMode === "create") {
        await invoke("create_tag", {
          projectId,
          name: editingTag.name.trim(),
          color: editingTag.color || null,
          parentId: editingTag.parent_id || null,
        });
      } else if (editingTag.id) {
        await invoke("update_tag", {
          tagId: editingTag.id,
          update: {
            name: editingTag.name.trim(),
            color: editingTag.color !== undefined ? editingTag.color : undefined,
            parent_id: undefined,
            position: undefined,
          },
        });
      }
      editingTag = null;
      await loadTags();
      onChange?.();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  async function deleteTag(id: string) {
    if (saving) return;
    saving = true;
    try {
      await invoke("delete_tag", { tagId: id });
      await loadTags();
      onChange?.();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }
</script>

{#snippet tagRow(tag: Tag, depth: number)}
  {@const children = getChildTags(tag.id)}
  <li>
    <div class="ka-tagrow">
      <span
        class="ka-swatch"
        class:ka-swatch--none={!tag.color}
        style:--swatch={tagColor(tag.color) ?? "transparent"}
        aria-hidden="true"
      ></span>
      <span class="ka-name">{tag.name}</span>
      {#if children.length > 0}
        <span class="ka-badge" title={`${children.length} child tags`}>{children.length}</span>
      {/if}
      <div class="ka-row">
        {#if depth < 2}
          <button
            type="button"
            onclick={() => openCreateForm(tag.id)}
            class="ka-button ka-button--ghost ka-icon-button"
            aria-label="Add child tag"
            title="Add child tag"
            disabled={!!editingTag}
          >
            <Plus class="w-5 h-5" aria-hidden="true" />
          </button>
        {/if}
        <button
          type="button"
          onclick={() => openEditForm(tag)}
          class="ka-button ka-button--ghost ka-icon-button"
          aria-label="Edit tag"
          title="Edit tag"
          disabled={!!editingTag}
        >
          <Pencil class="w-5 h-5" aria-hidden="true" />
        </button>
        <button
          type="button"
          onclick={() => deleteTag(tag.id)}
          class="ka-button ka-button--ghost ka-icon-button tag-delete"
          aria-label="Delete tag"
          title="Delete tag"
        >
          <Trash2 class="w-5 h-5" aria-hidden="true" />
        </button>
      </div>
    </div>
    {#if children.length > 0}
      <ul>
        {#each children as child (child.id)}
          {@render tagRow(child, depth + 1)}
        {/each}
      </ul>
    {/if}
  </li>
{/snippet}

<div class="tags">
  <div class="tags-head">
    <p class="ka-help">Tags nest up to three levels.</p>
    <button
      type="button"
      onclick={() => openCreateForm()}
      class="ka-button ka-button--secondary"
      disabled={!!editingTag}
    >
      <Plus class="w-5 h-5" aria-hidden="true" />
      New tag
    </button>
  </div>

  {#if loading}
    <p class="ka-help" role="status">Loading tags…</p>
  {:else if error}
    <p class="ka-error" role="alert">{error}</p>
  {:else if tags.length === 0 && !editingTag}
    <p class="ka-help">No tags defined yet.</p>
  {:else}
    <ul class="ka-tagtree">
      {#each getRootTags() as tag (tag.id)}
        {@render tagRow(tag, 0)}
      {/each}
    </ul>
  {/if}

  {#if editingTag}
    <div class="ka-tagedit">
      <div class="tags-edit-head">
        <h4>{editMode === "create" ? "New tag" : "Edit tag"}</h4>
        <button
          type="button"
          onclick={cancelEdit}
          class="ka-button ka-button--ghost ka-icon-button"
          aria-label="Close tag editor"
          title="Close tag editor"
        >
          <X class="w-5 h-5" aria-hidden="true" />
        </button>
      </div>

      <div class="ka-field od-field">
        <label for="tag-name">Name</label>
        <input
          id="tag-name"
          type="text"
          use:focusOnMount
          bind:value={editingTag.name}
          placeholder="e.g. Flashback, Action, Romance…"
          disabled={saving}
        />
      </div>

      <fieldset class="ka-swatches">
        <legend>Color</legend>
        <button
          type="button"
          onclick={() => {
            if (editingTag) editingTag.color = null;
          }}
          class="ka-swatch-option ka-swatch-option--none swatch-button"
          class:is-selected={!editingTag.color}
          aria-pressed={!editingTag.color}
          aria-label="No color"
          title="No color"
        >
          <span
            >{#if !editingTag.color}<Check class="w-4 h-4" aria-hidden="true" />{/if}</span
          >
        </button>
        {#each PRESET_COLORS as preset (preset.name)}
          {@const color = preset.color}
          <button
            type="button"
            onclick={() => {
              if (editingTag) editingTag.color = color;
            }}
            class="ka-swatch-option swatch-button"
            class:is-selected={editingTag.color === color}
            style:--swatch={tagColor(color)}
            aria-pressed={editingTag.color === color}
            aria-label={preset.name}
            title={preset.name}
          >
            <span
              >{#if editingTag.color === color}<Check
                  class="w-4 h-4"
                  aria-hidden="true"
                />{/if}</span
            >
          </button>
        {/each}
      </fieldset>

      <div class="ka-row tags-edit-actions">
        <button
          type="button"
          onclick={cancelEdit}
          class="ka-button ka-button--secondary"
          disabled={saving}
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={saveTag}
          class="ka-button"
          disabled={saving || !editingTag.name?.trim()}
          aria-busy={saving || undefined}
        >
          {#if saving}
            <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
          {/if}
          {editMode === "create" ? "Add" : "Save"}
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .tags {
    display: grid;
    gap: var(--space-s);
  }
  .tags-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-s);
  }
  .tags-head .ka-help {
    margin: 0;
  }
  .tags-edit-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: calc(-1 * var(--space-2xs)) calc(-1 * var(--space-2xs)) 0 0;
  }
  .tags-edit-head h4 {
    margin: 0;
    font: 600 var(--text-base) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .tags-edit-actions {
    justify-content: flex-end;
  }
  .swatch-button {
    padding: 0;
    border: 0;
    background: transparent;
  }
  .swatch-button.is-selected > span {
    outline: 2px solid var(--color-text);
    outline-offset: 2px;
  }
  .swatch-button:focus-visible {
    outline: 2px solid var(--color-accent-text);
    outline-offset: 1px;
  }
  @media (hover: hover) {
    .tag-delete:hover {
      color: var(--color-error);
    }
  }
</style>
