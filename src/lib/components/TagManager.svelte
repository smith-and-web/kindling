<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Loader2, Pencil, Plus, Trash2, X, Check } from "lucide-svelte";
  import pressTokens from "../../styles/press/tokens.json";
  import type { Tag } from "../types";
  import Tooltip from "./Tooltip.svelte";

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

  const inputClass =
    "w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2 text-press-ui focus:outline-none focus:border-press-accent";
</script>

<div class="space-y-3">
  <div class="flex items-center justify-between">
    <h3 class="text-press-ui font-medium text-press-text">Tags</h3>
    <button
      type="button"
      onclick={() => openCreateForm()}
      class="text-press-muted hover:text-press-text text-press-eyebrow flex items-center gap-1"
      disabled={!!editingTag}
    >
      <Plus class="w-3 h-3" />
      New tag
    </button>
  </div>

  {#if loading}
    <p class="text-press-eyebrow text-press-muted">Loading tags...</p>
  {:else if error}
    <p class="text-press-eyebrow text-press-error">{error}</p>
  {:else if tags.length === 0 && !editingTag}
    <p class="text-press-eyebrow text-press-muted">No tags defined yet.</p>
  {:else}
    <div class="space-y-0.5">
      {#each getRootTags() as tag}
        {@const children = getChildTags(tag.id)}
        <div>
          <div
            class="flex items-center gap-2 py-1.5 px-2 bg-press-sunken rounded-lg text-press-ui group"
          >
            {#if tag.color}
              <span class="w-3 h-3 rounded-full shrink-0" style:background-color={tag.color}></span>
            {:else}
              <span class="w-3 h-3 rounded-full shrink-0 bg-press-border"></span>
            {/if}
            <span class="flex-1 text-press-text truncate">{tag.name}</span>
            {#if children.length > 0}
              <span class="text-press-eyebrow text-press-muted">{children.length}</span>
            {/if}
            <div
              class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity"
            >
              <Tooltip text="Add child" position="bottom">
                <button
                  onclick={() => openCreateForm(tag.id)}
                  class="p-0.5 text-press-muted hover:text-press-text"
                  aria-label="Add child tag"
                >
                  <Plus class="w-3 h-3" />
                </button>
              </Tooltip>
              <Tooltip text="Edit" position="bottom">
                <button
                  onclick={() => openEditForm(tag)}
                  class="p-0.5 text-press-muted hover:text-press-text"
                  aria-label="Edit tag"
                >
                  <Pencil class="w-3 h-3" />
                </button>
              </Tooltip>
              <Tooltip text="Delete" position="bottom">
                <button
                  onclick={() => deleteTag(tag.id)}
                  class="p-0.5 text-press-muted hover:text-press-error"
                  aria-label="Delete tag"
                >
                  <Trash2 class="w-3 h-3" />
                </button>
              </Tooltip>
            </div>
          </div>
          {#if children.length > 0}
            <div class="ml-4 mt-0.5 space-y-0.5">
              {#each children as child}
                {@const grandchildren = getChildTags(child.id)}
                <div>
                  <div
                    class="flex items-center gap-2 py-1 px-2 bg-press-sunken rounded text-press-ui group"
                  >
                    {#if child.color}
                      <span
                        class="w-2.5 h-2.5 rounded-full shrink-0"
                        style:background-color={child.color}
                      ></span>
                    {:else}
                      <span class="w-2.5 h-2.5 rounded-full shrink-0 bg-press-border"></span>
                    {/if}
                    <span class="flex-1 text-press-text truncate text-press-eyebrow"
                      >{child.name}</span
                    >
                    <div
                      class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity"
                    >
                      {#if grandchildren.length === 0}
                        <Tooltip text="Add child" position="bottom">
                          <button
                            onclick={() => openCreateForm(child.id)}
                            class="p-0.5 text-press-muted hover:text-press-text"
                            aria-label="Add child tag"
                          >
                            <Plus class="w-3 h-3" />
                          </button>
                        </Tooltip>
                      {/if}
                      <Tooltip text="Edit" position="bottom">
                        <button
                          onclick={() => openEditForm(child)}
                          class="p-0.5 text-press-muted hover:text-press-text"
                          aria-label="Edit tag"
                        >
                          <Pencil class="w-3 h-3" />
                        </button>
                      </Tooltip>
                      <Tooltip text="Delete" position="bottom">
                        <button
                          onclick={() => deleteTag(child.id)}
                          class="p-0.5 text-press-muted hover:text-press-error"
                          aria-label="Delete tag"
                        >
                          <Trash2 class="w-3 h-3" />
                        </button>
                      </Tooltip>
                    </div>
                  </div>
                  {#if grandchildren.length > 0}
                    <div class="ml-4 mt-0.5 space-y-0.5">
                      {#each grandchildren as gc}
                        <div
                          class="flex items-center gap-2 py-1 px-2 bg-press-sunken rounded text-press-eyebrow group"
                        >
                          {#if gc.color}
                            <span
                              class="w-2 h-2 rounded-full shrink-0"
                              style:background-color={gc.color}
                            ></span>
                          {:else}
                            <span class="w-2 h-2 rounded-full shrink-0 bg-press-border"></span>
                          {/if}
                          <span class="flex-1 text-press-text truncate">{gc.name}</span>
                          <div
                            class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity"
                          >
                            <Tooltip text="Edit" position="bottom">
                              <button
                                onclick={() => openEditForm(gc)}
                                class="p-0.5 text-press-muted hover:text-press-text"
                                aria-label="Edit tag"
                              >
                                <Pencil class="w-3 h-3" />
                              </button>
                            </Tooltip>
                            <Tooltip text="Delete" position="bottom">
                              <button
                                onclick={() => deleteTag(gc.id)}
                                class="p-0.5 text-press-muted hover:text-press-error"
                                aria-label="Delete tag"
                              >
                                <Trash2 class="w-3 h-3" />
                              </button>
                            </Tooltip>
                          </div>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  {#if editingTag}
    <div class="bg-press-sunken rounded-lg p-3 space-y-3 border border-press-accent">
      <div class="flex items-center justify-between">
        <span class="text-press-ui font-medium text-press-text">
          {editMode === "create" ? "New Tag" : "Edit Tag"}
        </span>
        <button onclick={cancelEdit} class="p-1 text-press-muted hover:text-press-text">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div>
        <label class="block text-press-eyebrow text-press-muted mb-1" for="tag-name">Name</label>
        <input
          id="tag-name"
          type="text"
          bind:value={editingTag.name}
          class={inputClass}
          placeholder="e.g. Flashback, Action, Romance..."
          disabled={saving}
        />
      </div>

      <fieldset>
        <legend class="block text-press-eyebrow text-press-muted mb-1">Color</legend>
        <div class="flex flex-wrap gap-1.5">
          <button
            onclick={() => {
              if (editingTag) editingTag.color = null;
            }}
            class="w-6 h-6 rounded-full border-2 flex items-center justify-center"
            class:border-press-accent={!editingTag.color}
            class:border-transparent={!!editingTag.color}
            style:background-color="var(--color-surface-sunken)"
            aria-label="No color"
          >
            {#if !editingTag.color}
              <Check class="w-3 h-3 text-press-muted" />
            {/if}
          </button>
          {#each PRESET_COLORS as preset (preset.name)}
            {@const color = preset.color}
            <button
              onclick={() => {
                if (editingTag) editingTag.color = color;
              }}
              class="w-6 h-6 rounded-full border-2 flex items-center justify-center"
              class:border-press-on-accent={editingTag.color === color}
              class:border-transparent={editingTag.color !== color}
              style:background-color={color}
              aria-label={preset.name}
            >
              {#if editingTag.color === color}
                <Check class="w-3 h-3 text-press-on-accent" />
              {/if}
            </button>
          {/each}
        </div>
      </fieldset>

      <div class="flex justify-end gap-2">
        <button
          onclick={cancelEdit}
          class="px-3 py-1.5 text-press-ui text-press-muted hover:text-press-text"
          disabled={saving}
        >
          Cancel
        </button>
        <button
          onclick={saveTag}
          class="px-3 py-1.5 text-press-ui bg-press-accent text-press-on-accent rounded-lg hover:bg-press-accent-text flex items-center gap-1.5"
          disabled={saving || !editingTag.name?.trim()}
        >
          {#if saving}
            <Loader2 class="w-3.5 h-3.5 animate-spin" />
          {:else}
            <Check class="w-3.5 h-3.5" />
          {/if}
          {editMode === "create" ? "Add" : "Save"}
        </button>
      </div>
    </div>
  {/if}
</div>
