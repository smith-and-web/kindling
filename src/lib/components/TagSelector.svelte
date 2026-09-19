<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Plus, X } from "lucide-svelte";
  import type { Tag } from "../types";

  let {
    projectId,
    entityType,
    entityId,
    allTags = [],
    entityTagIds = [],
    onTagsChanged,
  }: {
    projectId: string;
    entityType: string;
    entityId: string;
    allTags: Tag[];
    entityTagIds: string[];
    onTagsChanged?: () => void;
  } = $props();

  let showDropdown = $state(false);
  let search = $state("");
  let creating = $state(false);

  let appliedTags = $derived(allTags.filter((t) => entityTagIds.includes(t.id)));
  let availableTags = $derived(
    allTags
      .filter((t) => !entityTagIds.includes(t.id))
      .filter((t) => !search || t.name.toLowerCase().includes(search.toLowerCase()))
  );

  async function addTag(tag: Tag) {
    try {
      await invoke("tag_entity", { tagId: tag.id, entityType, entityId });
      onTagsChanged?.();
    } catch (e) {
      console.error("Failed to add tag:", e);
    }
  }

  async function removeTag(tagId: string) {
    try {
      await invoke("untag_entity", { tagId, entityType, entityId });
      onTagsChanged?.();
    } catch (e) {
      console.error("Failed to remove tag:", e);
    }
  }

  async function createAndAdd() {
    const name = search.trim();
    if (!name) return;
    creating = true;
    try {
      const newTag = await invoke<Tag>("create_tag", {
        projectId,
        name,
        color: null,
        parentId: null,
      });
      await invoke("tag_entity", { tagId: newTag.id, entityType, entityId });
      search = "";
      onTagsChanged?.();
    } catch (e) {
      console.error("Failed to create tag:", e);
    } finally {
      creating = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      showDropdown = false;
    } else if (e.key === "Enter" && search.trim() && availableTags.length === 0) {
      createAndAdd();
    }
  }

  function handleClickOutside(e: MouseEvent) {
    if (showDropdown && dropdownRef && !dropdownRef.contains(e.target as Node)) {
      showDropdown = false;
    }
  }

  let dropdownRef: HTMLDivElement | null = $state(null);

  $effect(() => {
    if (showDropdown) {
      document.addEventListener("mousedown", handleClickOutside);
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }
  });
</script>

<div class="inline-flex flex-wrap gap-1 items-center">
  {#each appliedTags as tag}
    <span
      class="inline-flex items-center gap-1 text-press-eyebrow text-press-text px-1.5 py-0.5 rounded-full border"
      style:background-color={tag.color
        ? `color-mix(in srgb, ${tag.color} 12%, transparent)`
        : "var(--color-surface-sunken)"}
      style:border-color={tag.color || "var(--color-border)"}
    >
      {tag.name}
      <button
        onclick={() => removeTag(tag.id)}
        class="hover:text-press-error"
        aria-label="Remove tag {tag.name}"
      >
        <X class="w-3 h-3" />
      </button>
    </span>
  {/each}

  <div class="relative" bind:this={dropdownRef}>
    <button
      onclick={() => (showDropdown = !showDropdown)}
      class="inline-flex items-center gap-0.5 text-press-eyebrow text-press-muted hover:text-press-text px-1.5 py-0.5 rounded border border-dashed border-press-border hover:border-press-accent"
      aria-label="Add tag"
    >
      <Plus class="w-3 h-3" />
      Tag
    </button>

    {#if showDropdown}
      <div
        class="absolute top-full left-0 mt-1 w-48 bg-press-surface rounded-lg shadow-press-overlay border border-press-border z-press-dropdown overflow-hidden"
      >
        <div class="p-1.5">
          <input
            type="text"
            bind:value={search}
            onkeydown={handleKeydown}
            placeholder="Search or create..."
            class="w-full bg-press-sunken text-press-text text-press-eyebrow rounded px-2 py-1.5 focus:outline-none focus:ring-1 focus:ring-press-focus"
          />
        </div>
        <div class="max-h-40 overflow-y-auto">
          {#each availableTags as tag}
            <button
              onclick={() => {
                addTag(tag);
                showDropdown = false;
              }}
              class="w-full text-left px-2 py-1.5 text-press-eyebrow text-press-text hover:bg-press-sunken flex items-center gap-2"
            >
              {#if tag.color}
                <span class="w-2.5 h-2.5 rounded-full shrink-0" style:background-color={tag.color}
                ></span>
              {/if}
              {tag.name}
            </button>
          {/each}
          {#if search.trim() && availableTags.length === 0}
            <button
              onclick={createAndAdd}
              class="w-full text-left px-2 py-1.5 text-press-eyebrow text-press-accent-text hover:bg-press-sunken"
              disabled={creating}
            >
              Create "{search.trim()}"
            </button>
          {/if}
          {#if !search.trim() && availableTags.length === 0}
            <p class="px-2 py-1.5 text-press-eyebrow text-press-muted">No more tags available</p>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>
