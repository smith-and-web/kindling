<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { BookOpen, ChevronDown, ChevronRight, Layout, Loader2, X } from "lucide-svelte";
  import type { ProjectType, StoryTemplate } from "../types";
  import Tooltip from "./Tooltip.svelte";

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

  $effect(() => {
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
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="template-browser-title"
  tabindex="-1"
>
  <div
    class="bg-bg-panel rounded-lg shadow-xl w-full max-w-2xl mx-4 overflow-hidden max-h-[80vh] flex flex-col"
  >
    <div class="flex items-center justify-between px-4 py-3 border-b border-bg-card shrink-0">
      <h2 id="template-browser-title" class="text-lg font-medium text-text-primary">
        Story Structure Templates
      </h2>
      <Tooltip text="Close" position="left">
        <button
          type="button"
          onclick={onClose}
          class="p-1 text-text-secondary hover:text-text-primary transition-colors rounded"
          aria-label="Close"
        >
          <X class="w-5 h-5" />
        </button>
      </Tooltip>
    </div>

    <div class="flex-1 overflow-y-auto p-4">
      {#if loading}
        <div class="flex items-center justify-center py-12">
          <Loader2 class="w-6 h-6 animate-spin text-text-secondary" />
        </div>
      {:else if filteredTemplates.length === 0}
        <p class="text-text-secondary text-center py-8">No templates available.</p>
      {:else}
        <div class="space-y-2">
          {#each filteredTemplates as template}
            <div
              class="rounded-lg border-2 transition-colors {selectedId === template.id
                ? 'border-accent bg-accent/5'
                : 'border-bg-card hover:border-accent/30'}"
            >
              <button
                type="button"
                onclick={() => {
                  selectedId = template.id;
                  toggleExpand(template.id);
                }}
                class="w-full text-left px-4 py-3"
              >
                <div class="flex items-start gap-3">
                  <div
                    class="shrink-0 mt-0.5 p-1.5 rounded-lg {template.bundled
                      ? 'bg-accent/10 text-accent'
                      : 'bg-bg-card text-text-secondary'}"
                  >
                    {#if template.bundled}
                      <Layout class="w-4 h-4" />
                    {:else}
                      <BookOpen class="w-4 h-4" />
                    {/if}
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <span class="font-medium text-text-primary">{template.name}</span>
                      <span class="text-xs text-text-secondary">{totalBeats(template)} beats</span>
                      {#if template.source}
                        <span class="text-xs text-text-secondary">· {template.source}</span>
                      {/if}
                    </div>
                    {#if template.description}
                      <p class="text-sm text-text-secondary mt-1 line-clamp-2">
                        {template.description}
                      </p>
                    {/if}
                  </div>
                  <div class="shrink-0 mt-1 text-text-secondary">
                    {#if expandedId === template.id}
                      <ChevronDown class="w-4 h-4" />
                    {:else}
                      <ChevronRight class="w-4 h-4" />
                    {/if}
                  </div>
                </div>
              </button>

              {#if expandedId === template.id}
                <div class="px-4 pb-3 ml-10">
                  <div class="border-l-2 border-bg-card pl-3 space-y-1">
                    {#each template.structure as part}
                      <div>
                        <p class="text-xs font-medium text-accent">{part.title}</p>
                        {#each part.children as chapter}
                          <div class="ml-3 text-xs text-text-secondary py-0.5">
                            {chapter.title}
                            {#if chapter.synopsis}
                              <span class="text-text-secondary/60"> — {chapter.synopsis}</span>
                            {/if}
                          </div>
                        {/each}
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="flex items-center justify-between px-4 py-3 border-t border-bg-card shrink-0">
      <p class="text-xs text-text-secondary">
        {filteredTemplates.length} template{filteredTemplates.length !== 1 ? "s" : ""} available
      </p>
      <div class="flex items-center gap-2">
        <button
          type="button"
          onclick={onClose}
          class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleSelect}
          disabled={!selectedTemplate}
          class="px-4 py-2 text-sm bg-accent text-white rounded-lg hover:bg-accent/80 transition-colors disabled:opacity-50"
        >
          Use Template
        </button>
      </div>
    </div>
  </div>
</div>
