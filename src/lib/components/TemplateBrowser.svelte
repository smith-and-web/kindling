<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { BookOpen, Check, ChevronRight, Layout } from "lucide-svelte";
  import type { ProjectType, StoryTemplate } from "../types";
  import DialogHeader from "./DialogHeader.svelte";

  let {
    projectType = "novel",
    onSelect,
    onClose,
  }: {
    projectType?: ProjectType;
    onSelect: (template: StoryTemplate) => void;
    onClose: () => void;
  } = $props();

  let templates = $state<StoryTemplate[]>([]);
  let loading = $state(true);
  let selectedId = $state<string | null>(null);
  let expandedId = $state<string | null>(null);

  const filteredTemplates = $derived(
    templates.filter((t) => t.project_types.includes(projectType))
  );

  const selectedTemplate = $derived(filteredTemplates.find((t) => t.id === selectedId) ?? null);

  onMount(() => {
    loadTemplates();
  });

  async function loadTemplates() {
    loading = true;
    try {
      const [bundled, user] = await Promise.all([
        invoke<StoryTemplate[]>("get_bundled_templates"),
        invoke<StoryTemplate[]>("get_user_templates", { projectId: null }),
      ]);
      templates = [...bundled, ...user];
    } catch (e) {
      console.error("Failed to load templates:", e);
    } finally {
      loading = false;
    }
  }

  function toggleExpand(id: string) {
    expandedId = expandedId === id ? null : id;
  }

  function handleSelect() {
    if (selectedTemplate) {
      onSelect(selectedTemplate);
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") onClose();
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) onClose();
  }

  function totalBeats(template: StoryTemplate): number {
    return template.structure.reduce((sum, part) => sum + part.children.length, 0);
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="dialog-scrim"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="template-browser-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-default dialog-shell templates">
    <DialogHeader
      title="Story structure templates"
      titleId="template-browser-title"
      {onClose}
      closeLabel="Close"
      closeTestId="template-close"
    />

    <div class="ka-dialog-body">
      {#if loading}
        <div class="ka-progress od-field" role="status">
          <span>Loading templates…</span>
          <progress aria-label="Loading templates"></progress>
        </div>
      {:else if filteredTemplates.length === 0}
        <div class="ka-empty od-stack">
          <h4>No templates available</h4>
          <p>You can still start from a blank structure and add chapters yourself.</p>
        </div>
      {:else}
        <ul class="template-list">
          {#each filteredTemplates as template}
            {@const selected = selectedId === template.id}
            {@const expanded = expandedId === template.id}
            <li class="template" class:is-selected={selected}>
              <button
                type="button"
                onclick={() => {
                  selectedId = template.id;
                  toggleExpand(template.id);
                }}
                class="template-main"
                aria-pressed={selected}
                aria-expanded={expanded}
              >
                <span class="template-icon" aria-hidden="true">
                  {#if template.bundled}
                    <Layout class="w-5 h-5" />
                  {:else}
                    <BookOpen class="w-5 h-5" />
                  {/if}
                </span>
                <span class="template-text">
                  <span class="template-name">{template.name}</span>
                  <small>
                    {totalBeats(template)} beats{#if template.source}
                      · {template.source}{/if}
                  </small>
                  {#if template.description}
                    <span class="template-desc">{template.description}</span>
                  {/if}
                </span>
                {#if selected}
                  <Check class="w-5 h-5 template-check" aria-hidden="true" />
                {/if}
                <ChevronRight class="w-5 h-5 template-chev" aria-hidden="true" />
              </button>

              {#if expanded}
                <div class="template-structure">
                  {#each template.structure as part}
                    <div>
                      <p class="template-part">{part.title}</p>
                      <ul>
                        {#each part.children as chapter}
                          <li>
                            {chapter.title}
                            {#if chapter.synopsis}
                              <span> — {chapter.synopsis}</span>
                            {/if}
                          </li>
                        {/each}
                      </ul>
                    </div>
                  {/each}
                </div>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    <footer class="ka-dialog-footer">
      <p class="ka-help ka-dialog-footer-start">
        {filteredTemplates.length} template{filteredTemplates.length !== 1 ? "s" : ""} available
      </p>
      <button type="button" onclick={onClose} class="ka-button ka-button--secondary">
        Cancel
      </button>
      <button type="button" onclick={handleSelect} disabled={!selectedTemplate} class="ka-button">
        Use template
      </button>
    </footer>
  </div>
</div>

<style>
  .templates {
    height: min(720px, calc(100dvh - 48px));
  }
  .template-list {
    display: grid;
    margin: 0;
    padding: 0;
    list-style: none;
    border-top: var(--border-hair);
  }
  .template {
    border-bottom: var(--border-hair);
  }
  .template-main {
    display: flex;
    align-items: flex-start;
    gap: var(--space-xs);
    width: 100%;
    min-height: 60px;
    padding: var(--space-xs) var(--space-2xs);
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--color-text);
    font: var(--text-ui) / 1.5 var(--font-ui);
    text-align: left;
    cursor: pointer;
  }
  @media (hover: hover) {
    .template-main:hover {
      background: var(--color-surface-sunken);
    }
  }
  .template.is-selected .template-main {
    box-shadow: inset 3px 0 0 var(--color-accent-text);
  }
  .template-icon {
    display: grid;
    place-items: center;
    flex: none;
    width: 32px;
    height: 32px;
    border: var(--border-hair);
    border-radius: var(--radius-s);
    color: var(--color-text-muted);
  }
  .template-text {
    display: grid;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }
  .template-name {
    font-weight: 500;
  }
  .template-text small,
  .template-desc {
    font-size: var(--text-small);
    color: var(--color-text-muted);
  }
  .template-desc {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .templates :global(.template-check) {
    flex: none;
    margin-top: 6px;
    color: var(--color-accent-text);
  }
  .templates :global(.template-chev) {
    flex: none;
    margin-top: 6px;
    color: var(--color-text-muted);
    transition: transform var(--ka-motion) ease-out;
  }
  .template-main[aria-expanded="true"] :global(.template-chev) {
    transform: rotate(90deg);
  }
  .template-structure {
    display: grid;
    gap: var(--space-2xs);
    margin: 0 0 var(--space-xs) 52px;
    padding-left: var(--space-xs);
    border-left: var(--border-hair);
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .template-part {
    margin: 0;
    font-weight: 600;
    color: var(--color-text);
  }
  .template-structure ul {
    margin: 0;
    padding-left: var(--space-xs);
    list-style: none;
  }
</style>
