<script lang="ts">
  import { tagColor } from "../utils/tagColor";
  import { invoke } from "@tauri-apps/api/core";
  import { Plus, X } from "lucide-svelte";
  import type { Tag } from "../types";
  import { ui } from "../stores/ui.svelte";

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
      ui.showError(`Failed to add tag: ${String(e)}`);
    }
  }

  async function removeTag(tagId: string) {
    try {
      await invoke("untag_entity", { tagId, entityType, entityId });
      onTagsChanged?.();
    } catch (e) {
      console.error("Failed to remove tag:", e);
      ui.showError(`Failed to remove tag: ${String(e)}`);
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
      ui.showError(`Failed to create tag: ${String(e)}`);
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

<div class="tags">
  {#each appliedTags as tag}
    <span class="tag-chip" style:--tag={tagColor(tag.color) ?? "var(--color-border)"}>
      <span class="tag-dot" aria-hidden="true"></span>
      {tag.name}
      <button
        type="button"
        onclick={() => removeTag(tag.id)}
        class="tag-remove"
        aria-label="Remove tag {tag.name}"
        title="Remove tag"
      >
        <X class="w-4 h-4" aria-hidden="true" />
      </button>
    </span>
  {/each}

  <div class="relative" bind:this={dropdownRef}>
    <button
      type="button"
      onclick={() => (showDropdown = !showDropdown)}
      class="ka-button ka-button--ghost tag-add"
      aria-label="Add tag"
      aria-expanded={showDropdown}
    >
      <Plus class="w-4 h-4" aria-hidden="true" />
      Add tag
    </button>

    {#if showDropdown}
      <div class="app-popover tag-popover">
        <div class="tag-search">
          <input
            type="text"
            bind:value={search}
            onkeydown={handleKeydown}
            placeholder="Search or create…"
            aria-label="Search or create a tag"
          />
        </div>
        <div class="tag-options">
          {#each availableTags as tag}
            <button
              type="button"
              onclick={() => {
                addTag(tag);
                showDropdown = false;
              }}
              class="tag-option"
              style:--tag={tagColor(tag.color) ?? "transparent"}
            >
              <span class="tag-dot" aria-hidden="true"></span>
              {tag.name}
            </button>
          {/each}
          {#if search.trim() && availableTags.length === 0}
            <button
              type="button"
              onclick={createAndAdd}
              class="tag-option tag-create"
              disabled={creating}
              aria-busy={creating || undefined}
            >
              <Plus class="w-4 h-4" aria-hidden="true" />
              Create “{search.trim()}”
            </button>
          {/if}
          {#if !search.trim() && availableTags.length === 0}
            <p class="ka-help tag-none">No more tags available</p>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .tags {
    display: inline-flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2xs);
  }
  /* A tag is user metadata: its colour marks the dot and the rim, never the
     text, so every tag stays legible in either theme. */
  .tag-chip {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2xs);
    min-height: 32px;
    padding: 0 var(--space-3xs) 0 var(--space-xs);
    border: 1px solid color-mix(in srgb, var(--tag) 45%, var(--color-border));
    border-radius: var(--radius-pill);
    background: color-mix(in srgb, var(--tag) 10%, var(--color-surface));
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .tag-dot {
    width: 8px;
    height: 8px;
    flex: none;
    border-radius: 50%;
    background: var(--tag);
  }
  .tag-remove {
    position: relative;
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }
  .tag-remove::after {
    content: "";
    position: absolute;
    inset: -8px;
  }
  @media (hover: hover) {
    .tag-remove:hover {
      background: var(--color-error-wash);
      color: var(--color-error);
    }
  }
  .tag-add {
    padding: var(--space-2xs) var(--space-xs);
    font-size: var(--text-small);
    color: var(--color-text-muted);
  }
  .tag-popover {
    position: absolute;
    top: calc(100% + var(--space-3xs));
    left: 0;
    z-index: var(--z-dropdown);
    width: 240px;
    overflow: hidden;
  }
  .tag-search {
    padding: var(--space-2xs);
    border-bottom: var(--border-hair);
  }
  .tag-search input {
    width: 100%;
    min-height: var(--control-target);
    padding: var(--space-2xs) var(--space-xs);
    font-size: var(--text-ui);
  }
  .tag-options {
    display: grid;
    max-height: 13rem;
    overflow-y: auto;
    padding: var(--space-3xs);
  }
  .tag-option {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    min-height: var(--control-target);
    padding: var(--space-2xs) var(--space-xs);
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--color-text);
    font: var(--text-ui) / 1.5 var(--font-ui);
    text-align: left;
    cursor: pointer;
  }
  @media (hover: hover) {
    .tag-option:hover {
      background: var(--color-surface-sunken);
    }
  }
  .tag-create {
    color: var(--color-accent-text);
  }
  .tag-none {
    padding: var(--space-2xs) var(--space-xs);
  }
</style>
