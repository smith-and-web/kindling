<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Loader2, Pencil, Plus, Trash2, X, Check } from "lucide-svelte";
  import type { Tag } from "../types";
  import Tooltip from "./Tooltip.svelte";

  let {
    projectId,
  }: {
    projectId: string;
  } = $props();

  let tags = $state<Tag[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let editingTag = $state<Partial<Tag> | null>(null);
  let editMode = $state<"create" | "edit">("create");
  let saving = $state(false);

  const PRESET_COLORS = [
    "#ef4444",
    "#f97316",
    "#eab308",
    "#22c55e",
    "#06b6d4",
    "#3b82f6",
    "#8b5cf6",
    "#ec4899",
    "#6b7280",
    "#14b8a6",
  ];

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
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  async function deleteTag(id: string) {
    try {
      await invoke("delete_tag", { tagId: id });
      await loadTags();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  const inputClass =
    "w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-accent";
</script>

<div class="space-y-3">
  <div class="flex items-center justify-between">
    <h3 class="text-sm font-medium text-text-primary">Tags</h3>
    <button
      type="button"
      onclick={() => openCreateForm()}
      class="text-text-secondary hover:text-text-primary text-xs flex items-center gap-1"
      disabled={!!editingTag}
    >
      <Plus class="w-3 h-3" />
      New tag
    </button>
  </div>

  {#if loading}
    <p class="text-xs text-text-secondary">Loading tags...</p>
  {:else if error}
    <p class="text-xs text-red-400">{error}</p>
  {:else if tags.length === 0 && !editingTag}
    <p class="text-xs text-text-secondary">No tags defined yet.</p>
  {:else}
    <div class="space-y-0.5">
      {#each getRootTags() as tag (tag.id)}
        {@const children = getChildTags(tag.id)}
        <div>
          <div class="flex items-center gap-2 py-1.5 px-2 bg-bg-card rounded-lg text-sm group">
            {#if tag.color}
              <span class="w-3 h-3 rounded-full shrink-0" style:background-color={tag.color}></span>
            {:else}
              <span class="w-3 h-3 rounded-full shrink-0 bg-text-secondary/20"></span>
            {/if}
            <span class="flex-1 text-text-primary truncate">{tag.name}</span>
            {#if children.length > 0}
              <span class="text-xs text-text-secondary">{children.length}</span>
            {/if}
            <div
              class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity"
            >
              <Tooltip text="Add child" position="bottom">
                <button
                  onclick={() => openCreateForm(tag.id)}
                  class="p-0.5 text-text-secondary hover:text-text-primary"
                  aria-label="Add child tag"
                >
                  <Plus class="w-3 h-3" />
                </button>
              </Tooltip>
              <Tooltip text="Edit" position="bottom">
                <button
                  onclick={() => openEditForm(tag)}
                  class="p-0.5 text-text-secondary hover:text-text-primary"
                  aria-label="Edit tag"
                >
                  <Pencil class="w-3 h-3" />
                </button>
              </Tooltip>
              <Tooltip text="Delete" position="bottom">
                <button
                  onclick={() => deleteTag(tag.id)}
                  class="p-0.5 text-text-secondary hover:text-red-400"
                  aria-label="Delete tag"
                >
                  <Trash2 class="w-3 h-3" />
                </button>
              </Tooltip>
            </div>
          </div>
          {#if children.length > 0}
            <div class="ml-4 mt-0.5 space-y-0.5">
              {#each children as child (child.id)}
                {@const grandchildren = getChildTags(child.id)}
                <div>
                  <div
                    class="flex items-center gap-2 py-1 px-2 bg-bg-card/50 rounded text-sm group"
                  >
                    {#if child.color}
                      <span
                        class="w-2.5 h-2.5 rounded-full shrink-0"
                        style:background-color={child.color}
                      ></span>
                    {:else}
                      <span class="w-2.5 h-2.5 rounded-full shrink-0 bg-text-secondary/20"></span>
                    {/if}
                    <span class="flex-1 text-text-primary truncate text-xs">{child.name}</span>
                    <div
                      class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity"
                    >
                      {#if grandchildren.length === 0}
                        <Tooltip text="Add child" position="bottom">
                          <button
                            onclick={() => openCreateForm(child.id)}
                            class="p-0.5 text-text-secondary hover:text-text-primary"
                            aria-label="Add child tag"
                          >
                            <Plus class="w-3 h-3" />
                          </button>
                        </Tooltip>
                      {/if}
                      <Tooltip text="Edit" position="bottom">
                        <button
                          onclick={() => openEditForm(child)}
                          class="p-0.5 text-text-secondary hover:text-text-primary"
                          aria-label="Edit tag"
                        >
                          <Pencil class="w-3 h-3" />
                        </button>
                      </Tooltip>
                      <Tooltip text="Delete" position="bottom">
                        <button
                          onclick={() => deleteTag(child.id)}
                          class="p-0.5 text-text-secondary hover:text-red-400"
                          aria-label="Delete tag"
                        >
                          <Trash2 class="w-3 h-3" />
                        </button>
                      </Tooltip>
                    </div>
                  </div>
                  {#if grandchildren.length > 0}
                    <div class="ml-4 mt-0.5 space-y-0.5">
                      {#each grandchildren as gc (gc.id)}
                        <div
                          class="flex items-center gap-2 py-1 px-2 bg-bg-card/30 rounded text-xs group"
                        >
                          {#if gc.color}
                            <span
                              class="w-2 h-2 rounded-full shrink-0"
                              style:background-color={gc.color}
                            ></span>
                          {:else}
                            <span class="w-2 h-2 rounded-full shrink-0 bg-text-secondary/20"></span>
                          {/if}
                          <span class="flex-1 text-text-primary truncate">{gc.name}</span>
                          <div
                            class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity"
                          >
                            <Tooltip text="Edit" position="bottom">
                              <button
                                onclick={() => openEditForm(gc)}
                                class="p-0.5 text-text-secondary hover:text-text-primary"
                                aria-label="Edit tag"
                              >
                                <Pencil class="w-3 h-3" />
                              </button>
                            </Tooltip>
                            <Tooltip text="Delete" position="bottom">
                              <button
                                onclick={() => deleteTag(gc.id)}
                                class="p-0.5 text-text-secondary hover:text-red-400"
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
    <div class="bg-bg-card rounded-lg p-3 space-y-3 border border-accent/30">
      <div class="flex items-center justify-between">
        <span class="text-sm font-medium text-text-primary">
          {editMode === "create" ? "New Tag" : "Edit Tag"}
        </span>
        <button onclick={cancelEdit} class="p-1 text-text-secondary hover:text-text-primary">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div>
        <label class="block text-xs text-text-secondary mb-1" for="tag-name">Name</label>
        <input
          id="tag-name"
          type="text"
          bind:value={editingTag.name}
          class={inputClass}
          placeholder="e.g. Flashback, Action, Romance..."
          disabled={saving}
        />
      </div>

      <div>
        <label class="block text-xs text-text-secondary mb-1">Color</label>
        <div class="flex flex-wrap gap-1.5">
          <button
            onclick={() => {
              if (editingTag) editingTag.color = null;
            }}
            class="w-6 h-6 rounded-full border-2 flex items-center justify-center"
            class:border-accent={!editingTag.color}
            class:border-transparent={!!editingTag.color}
            style:background-color="var(--color-bg-card)"
            aria-label="No color"
          >
            {#if !editingTag.color}
              <Check class="w-3 h-3 text-text-secondary" />
            {/if}
          </button>
          {#each PRESET_COLORS as color}
            <button
              onclick={() => {
                if (editingTag) editingTag.color = color;
              }}
              class="w-6 h-6 rounded-full border-2 flex items-center justify-center"
              class:border-white={editingTag.color === color}
              class:border-transparent={editingTag.color !== color}
              style:background-color={color}
              aria-label="Color {color}"
            >
              {#if editingTag.color === color}
                <Check class="w-3 h-3 text-white" />
              {/if}
            </button>
          {/each}
        </div>
      </div>

      <div class="flex justify-end gap-2">
        <button
          onclick={cancelEdit}
          class="px-3 py-1.5 text-sm text-text-secondary hover:text-text-primary"
          disabled={saving}
        >
          Cancel
        </button>
        <button
          onclick={saveTag}
          class="px-3 py-1.5 text-sm bg-accent text-white rounded-lg hover:bg-accent/80 disabled:opacity-50 flex items-center gap-1.5"
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
