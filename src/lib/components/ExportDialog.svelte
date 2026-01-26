<!--
  ExportDialog.svelte - Export configuration dialog

  Allows users to configure and initiate project exports:
  - Format selection (Markdown or Word Document)
  - Scope selection based on context (project/chapter/scene)
  - Options like beat markers, synopsis, page breaks
  - Destination folder/file picker
-->
<script lang="ts">
  /* eslint-disable no-undef */
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import {
    X,
    Loader2,
    FolderOpen,
    FileText,
    FileDown,
    Type,
    AlignLeft,
    BookOpen,
    ChevronDown,
    Hash,
  } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import type {
    ExportResult,
    MarkdownExportOptions,
    DocxExportOptions,
    ExportScope,
    ChapterHeadingStyle,
    SceneBreakStyle,
    FontFamily,
    LineSpacingOption,
  } from "../types";
  import Tooltip from "./Tooltip.svelte";

  const LAST_EXPORT_PATH_KEY = "kindling:lastExportPath";

  let {
    scope,
    scopeId,
    scopeTitle,
    onClose,
    onSuccess,
  }: {
    scope: "project" | "chapter" | "scene";
    scopeId: string | null;
    scopeTitle: string;
    onClose: () => void;
    onSuccess: (result: ExportResult) => void;
  } = $props();

  let exportFormat = $state<"markdown" | "docx">("docx");
  let includeBeatMarkers = $state(false);
  let includeSynopsis = $state(false);
  let pageBreaksBetweenChapters = $state(true);
  let includeTitlePage = $state(true);
  let chapterHeadingStyle = $state<ChapterHeadingStyle>("number_only");
  let sceneBreakStyle = $state<SceneBreakStyle>("hash");
  let fontFamily = $state<FontFamily>("courier_new");
  let lineSpacing = $state<LineSpacingOption>("double");
  let deleteExisting = $state(false);
  let createSnapshot = $state(false);
  let outputPath = $state("");
  let docxFilePath = $state("");
  let exportName = $state("");
  let exporting = $state(false);
  let error = $state<string | null>(null);
  let wordCount = $state<number | null>(null);
  let loadingWordCount = $state(false);

  // Chapter heading style options for the dropdown
  const chapterHeadingStyles: { value: ChapterHeadingStyle; label: string; example: string }[] = [
    { value: "number_only", label: "Number Only", example: "CHAPTER ONE" },
    { value: "number_and_title", label: "Number and Title", example: "CHAPTER ONE: THE BEGINNING" },
    { value: "title_only", label: "Title Only", example: "THE BEGINNING" },
    { value: "number_arabic", label: "Arabic Numeral", example: "CHAPTER 1" },
    {
      value: "number_arabic_and_title",
      label: "Arabic and Title",
      example: "CHAPTER 1: THE BEGINNING",
    },
  ];

  // Scene break style options
  const sceneBreakStyles: { value: SceneBreakStyle; label: string; example: string }[] = [
    { value: "hash", label: "Hash Mark", example: "#" },
    { value: "asterisks", label: "Three Asterisks", example: "* * *" },
    { value: "asterism", label: "Asterism", example: "⁂" },
    { value: "blank_line", label: "Blank Line", example: "(blank)" },
  ];

  // Font family options
  const fontFamilies: { value: FontFamily; label: string }[] = [
    { value: "courier_new", label: "Courier New" },
    { value: "times_new_roman", label: "Times New Roman" },
  ];

  // Line spacing options
  const lineSpacingOptions: { value: LineSpacingOption; label: string }[] = [
    { value: "single", label: "Single" },
    { value: "one_and_half", label: "1.5 Lines" },
    { value: "double", label: "Double" },
  ];

  // Initialize export name from project name
  $effect(() => {
    if (currentProject.value && !exportName) {
      exportName = currentProject.value.name;
    }
  });

  // Load last export path from localStorage on mount
  $effect(() => {
    const savedPath = localStorage.getItem(LAST_EXPORT_PATH_KEY);
    if (savedPath && !outputPath) {
      outputPath = savedPath;
    }
  });

  // Fetch word count when dialog opens (for project-level export)
  $effect(() => {
    if (currentProject.value && scope === "project") {
      loadingWordCount = true;
      invoke<number>("get_project_word_count", {
        projectId: currentProject.value.id,
      })
        .then((count) => {
          wordCount = count;
        })
        .catch(() => {
          wordCount = null;
        })
        .finally(() => {
          loadingWordCount = false;
        });
    }
  });

  // Format word count for display (rounded to nearest 1000)
  const formattedWordCount = $derived(() => {
    if (wordCount === null) return null;
    if (wordCount < 1000) return `${wordCount} words`;
    const rounded = Math.round(wordCount / 1000) * 1000;
    return `~${rounded.toLocaleString()} words`;
  });

  const canExport = $derived(
    (exportFormat === "markdown" && outputPath.length > 0) ||
      (exportFormat === "docx" && docxFilePath.length > 0)
  );

  async function selectDestination() {
    const path = await open({
      directory: true,
      title: "Select Export Destination",
      defaultPath: outputPath || undefined,
    });

    if (path) {
      outputPath = path;
      error = null;
    }
  }

  async function selectDocxFile() {
    const defaultName = `${exportName.trim() || currentProject.value?.name || "Export"}.docx`;
    const path = await save({
      title: "Save Word Document",
      defaultPath: defaultName,
      filters: [{ name: "Word Document", extensions: ["docx"] }],
    });

    if (path) {
      docxFilePath = path;
      error = null;
    }
  }

  async function handleExport() {
    if (!canExport) return;

    exporting = true;
    error = null;

    try {
      // Build the scope for the export options
      let exportScope: ExportScope;
      if (scope === "project") {
        exportScope = "project";
      } else if (scope === "chapter" && scopeId) {
        exportScope = { chapter: scopeId };
      } else if (scope === "scene" && scopeId) {
        exportScope = { scene: scopeId };
      } else {
        exportScope = "project";
      }

      if (!currentProject.value) {
        throw new Error("No project selected");
      }

      let result: ExportResult;

      if (exportFormat === "markdown") {
        const options: MarkdownExportOptions = {
          scope: exportScope,
          include_beat_markers: includeBeatMarkers,
          output_path: outputPath,
          delete_existing: deleteExisting,
          export_name: exportName.trim() || undefined,
          create_snapshot: createSnapshot,
        };

        result = await invoke<ExportResult>("export_to_markdown", {
          projectId: currentProject.value.id,
          options,
        });

        // Save the export path for next time (markdown only, since it's a folder)
        localStorage.setItem(LAST_EXPORT_PATH_KEY, outputPath);
      } else {
        const options: DocxExportOptions = {
          scope: exportScope,
          include_beat_markers: includeBeatMarkers,
          include_synopsis: includeSynopsis,
          output_path: docxFilePath,
          create_snapshot: createSnapshot,
          page_breaks_between_chapters: pageBreaksBetweenChapters,
          include_title_page: includeTitlePage,
          chapter_heading_style: chapterHeadingStyle,
          scene_break_style: sceneBreakStyle,
          font_family: fontFamily,
          line_spacing: lineSpacing,
        };

        result = await invoke<ExportResult>("export_to_docx", {
          projectId: currentProject.value.id,
          options,
        });
      }

      onSuccess(result);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      exporting = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    } else if (event.key === "Enter" && canExport && !exporting) {
      handleExport();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Backdrop -->
<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="export-dialog-title"
  tabindex="-1"
>
  <!-- Dialog -->
  <div
    class="bg-bg-panel rounded-lg shadow-xl w-full max-w-lg mx-4 overflow-hidden max-h-[90vh] flex flex-col"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-5 py-4 border-b border-bg-card flex-shrink-0">
      <div class="flex items-center gap-3">
        <div class="p-2 bg-accent/10 rounded-lg">
          <FileDown class="w-5 h-5 text-accent" />
        </div>
        <div>
          <h2 id="export-dialog-title" class="text-lg font-medium text-text-primary">
            Export {scopeTitle}
          </h2>
          <p class="text-xs text-text-secondary">Choose format and configure options</p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        {#if scope === "project"}
          <div class="flex items-center gap-1.5 px-2.5 py-1.5 bg-bg-card/50 rounded-lg">
            <Hash class="w-3.5 h-3.5 text-text-secondary" />
            {#if loadingWordCount}
              <span class="text-xs text-text-secondary">...</span>
            {:else if formattedWordCount()}
              <span class="text-xs text-text-secondary">{formattedWordCount()}</span>
            {/if}
          </div>
        {/if}
        <Tooltip text="Close" position="left">
          <button
            type="button"
            onclick={onClose}
            class="p-2 text-text-secondary hover:text-text-primary hover:bg-bg-card transition-colors rounded-lg"
            aria-label="Close"
          >
            <X class="w-5 h-5" />
          </button>
        </Tooltip>
      </div>
    </div>

    <!-- Content -->
    <div class="p-5 space-y-5 overflow-y-auto flex-1">
      <!-- Format Selection - Card Style -->
      <fieldset>
        <legend class="block text-sm font-medium text-text-secondary mb-3">Export Format</legend>
        <div class="grid grid-cols-2 gap-3">
          <label
            class="relative flex flex-col items-center p-4 rounded-lg border-2 cursor-pointer transition-all {exportFormat ===
            'docx'
              ? 'border-accent bg-accent/5'
              : 'border-bg-card hover:border-text-secondary/30 bg-bg-card/50'}"
          >
            <input
              type="radio"
              name="format"
              value="docx"
              bind:group={exportFormat}
              class="sr-only"
            />
            <FileText
              class="w-8 h-8 mb-2 {exportFormat === 'docx' ? 'text-accent' : 'text-text-secondary'}"
            />
            <span
              class="text-sm font-medium {exportFormat === 'docx'
                ? 'text-text-primary'
                : 'text-text-secondary'}">Word Document</span
            >
            <span class="text-xs text-text-secondary mt-0.5">.docx</span>
            {#if exportFormat === "docx"}
              <div class="absolute top-2 right-2 w-2 h-2 rounded-full bg-accent"></div>
            {/if}
          </label>

          <label
            class="relative flex flex-col items-center p-4 rounded-lg border-2 cursor-pointer transition-all {exportFormat ===
            'markdown'
              ? 'border-accent bg-accent/5'
              : 'border-bg-card hover:border-text-secondary/30 bg-bg-card/50'}"
          >
            <input
              type="radio"
              name="format"
              value="markdown"
              bind:group={exportFormat}
              class="sr-only"
            />
            <BookOpen
              class="w-8 h-8 mb-2 {exportFormat === 'markdown'
                ? 'text-accent'
                : 'text-text-secondary'}"
            />
            <span
              class="text-sm font-medium {exportFormat === 'markdown'
                ? 'text-text-primary'
                : 'text-text-secondary'}">Markdown</span
            >
            <span class="text-xs text-text-secondary mt-0.5">.md files</span>
            {#if exportFormat === "markdown"}
              <div class="absolute top-2 right-2 w-2 h-2 rounded-full bg-accent"></div>
            {/if}
          </label>
        </div>
      </fieldset>

      {#if exportFormat === "docx"}
        <!-- DOCX Options Section -->
        <fieldset>
          <legend class="flex items-center gap-2 text-sm font-medium text-accent mb-3">
            <Type class="w-4 h-4" />
            Document Structure
          </legend>

          <!-- Toggle Options -->
          <div class="space-y-2 mb-4">
            <label
              class="flex items-center justify-between p-3 bg-bg-card/50 rounded-lg cursor-pointer hover:bg-bg-card transition-colors group"
            >
              <div class="flex items-center gap-3">
                <span class="text-sm text-text-primary">Include title page</span>
              </div>
              <div class="relative">
                <input type="checkbox" bind:checked={includeTitlePage} class="peer sr-only" />
                <div
                  class="w-10 h-6 bg-bg-card rounded-full peer-checked:bg-accent transition-colors"
                ></div>
                <div
                  class="absolute left-1 top-1 w-4 h-4 bg-text-secondary rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-white"
                ></div>
              </div>
            </label>

            <label
              class="flex items-center justify-between p-3 bg-bg-card/50 rounded-lg cursor-pointer hover:bg-bg-card transition-colors"
            >
              <span class="text-sm text-text-primary">Page breaks between chapters</span>
              <div class="relative">
                <input
                  type="checkbox"
                  bind:checked={pageBreaksBetweenChapters}
                  class="peer sr-only"
                />
                <div
                  class="w-10 h-6 bg-bg-card rounded-full peer-checked:bg-accent transition-colors"
                ></div>
                <div
                  class="absolute left-1 top-1 w-4 h-4 bg-text-secondary rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-white"
                ></div>
              </div>
            </label>

            <label
              class="flex items-center justify-between p-3 bg-bg-card/50 rounded-lg cursor-pointer hover:bg-bg-card transition-colors"
            >
              <span class="text-sm text-text-primary">Include beat markers as headings</span>
              <div class="relative">
                <input type="checkbox" bind:checked={includeBeatMarkers} class="peer sr-only" />
                <div
                  class="w-10 h-6 bg-bg-card rounded-full peer-checked:bg-accent transition-colors"
                ></div>
                <div
                  class="absolute left-1 top-1 w-4 h-4 bg-text-secondary rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-white"
                ></div>
              </div>
            </label>

            <label
              class="flex items-center justify-between p-3 bg-bg-card/50 rounded-lg cursor-pointer hover:bg-bg-card transition-colors"
            >
              <span class="text-sm text-text-primary">Include scene synopses</span>
              <div class="relative">
                <input type="checkbox" bind:checked={includeSynopsis} class="peer sr-only" />
                <div
                  class="w-10 h-6 bg-bg-card rounded-full peer-checked:bg-accent transition-colors"
                ></div>
                <div
                  class="absolute left-1 top-1 w-4 h-4 bg-text-secondary rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-white"
                ></div>
              </div>
            </label>
          </div>

          <!-- Dropdown Selects -->
          <div class="grid grid-cols-2 gap-3">
            <!-- Chapter Heading Style -->
            <div>
              <label for="chapter-heading-style" class="block text-xs text-text-secondary mb-1.5">
                Chapter Heading
              </label>
              <div class="relative">
                <select
                  id="chapter-heading-style"
                  bind:value={chapterHeadingStyle}
                  class="w-full appearance-none bg-bg-card text-text-primary text-sm border border-bg-card rounded-lg pl-3 pr-8 py-2.5 focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50 cursor-pointer"
                >
                  {#each chapterHeadingStyles as style (style.value)}
                    <option value={style.value}>{style.label}</option>
                  {/each}
                </select>
                <ChevronDown
                  class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-text-secondary pointer-events-none"
                />
              </div>
              <p class="text-xs text-text-secondary/70 mt-1 truncate">
                {chapterHeadingStyles.find((s) => s.value === chapterHeadingStyle)?.example}
              </p>
            </div>

            <!-- Scene Break Style -->
            <div>
              <label for="scene-break-style" class="block text-xs text-text-secondary mb-1.5">
                Scene Break
              </label>
              <div class="relative">
                <select
                  id="scene-break-style"
                  bind:value={sceneBreakStyle}
                  class="w-full appearance-none bg-bg-card text-text-primary text-sm border border-bg-card rounded-lg pl-3 pr-8 py-2.5 focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50 cursor-pointer"
                >
                  {#each sceneBreakStyles as style (style.value)}
                    <option value={style.value}>{style.label}</option>
                  {/each}
                </select>
                <ChevronDown
                  class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-text-secondary pointer-events-none"
                />
              </div>
              <p class="text-xs text-text-secondary/70 mt-1">
                {sceneBreakStyles.find((s) => s.value === sceneBreakStyle)?.example}
              </p>
            </div>
          </div>
        </fieldset>

        <!-- Typography Section -->
        <fieldset>
          <legend class="flex items-center gap-2 text-sm font-medium text-accent mb-3">
            <AlignLeft class="w-4 h-4" />
            Typography
          </legend>
          <div class="grid grid-cols-2 gap-3">
            <!-- Font Family -->
            <div>
              <label for="font-family" class="block text-xs text-text-secondary mb-1.5">
                Font
              </label>
              <div class="relative">
                <select
                  id="font-family"
                  bind:value={fontFamily}
                  class="w-full appearance-none bg-bg-card text-text-primary text-sm border border-bg-card rounded-lg pl-3 pr-8 py-2.5 focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50 cursor-pointer"
                >
                  {#each fontFamilies as font (font.value)}
                    <option value={font.value}>{font.label}</option>
                  {/each}
                </select>
                <ChevronDown
                  class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-text-secondary pointer-events-none"
                />
              </div>
            </div>

            <!-- Line Spacing -->
            <div>
              <label for="line-spacing" class="block text-xs text-text-secondary mb-1.5">
                Line Spacing
              </label>
              <div class="relative">
                <select
                  id="line-spacing"
                  bind:value={lineSpacing}
                  class="w-full appearance-none bg-bg-card text-text-primary text-sm border border-bg-card rounded-lg pl-3 pr-8 py-2.5 focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50 cursor-pointer"
                >
                  {#each lineSpacingOptions as spacing (spacing.value)}
                    <option value={spacing.value}>{spacing.label}</option>
                  {/each}
                </select>
                <ChevronDown
                  class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-text-secondary pointer-events-none"
                />
              </div>
            </div>
          </div>
        </fieldset>

        <!-- Save Location -->
        <div>
          <label for="docx-destination" class="block text-sm font-medium text-text-secondary mb-2">
            Save Location
          </label>
          <div class="flex gap-2">
            <input
              id="docx-destination"
              type="text"
              readonly
              value={docxFilePath}
              placeholder="Choose where to save..."
              class="flex-1 bg-bg-card text-text-primary text-sm border border-bg-card rounded-lg px-3 py-2.5 focus:outline-none focus:border-accent cursor-pointer truncate"
              onclick={selectDocxFile}
            />
            <Tooltip text="Browse" position="top">
              <button
                type="button"
                onclick={selectDocxFile}
                class="px-3 py-2.5 bg-bg-card text-text-secondary rounded-lg hover:bg-beat-header hover:text-text-primary transition-colors border border-bg-card"
                aria-label="Choose save location"
              >
                <FileText class="w-5 h-5" />
              </button>
            </Tooltip>
          </div>
        </div>
      {:else}
        <!-- Markdown Options -->
        <fieldset>
          <legend class="flex items-center gap-2 text-sm font-medium text-accent mb-3">
            <Type class="w-4 h-4" />
            Options
          </legend>

          <div class="space-y-2 mb-4">
            <label
              class="flex items-center justify-between p-3 bg-bg-card/50 rounded-lg cursor-pointer hover:bg-bg-card transition-colors"
            >
              <span class="text-sm text-text-primary">Include beat markers as headings</span>
              <div class="relative">
                <input type="checkbox" bind:checked={includeBeatMarkers} class="peer sr-only" />
                <div
                  class="w-10 h-6 bg-bg-card rounded-full peer-checked:bg-accent transition-colors"
                ></div>
                <div
                  class="absolute left-1 top-1 w-4 h-4 bg-text-secondary rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-white"
                ></div>
              </div>
            </label>

            <label
              class="flex items-center justify-between p-3 bg-bg-card/50 rounded-lg cursor-pointer hover:bg-bg-card transition-colors"
            >
              <span class="text-sm text-text-primary">Delete existing export folder</span>
              <div class="relative">
                <input type="checkbox" bind:checked={deleteExisting} class="peer sr-only" />
                <div
                  class="w-10 h-6 bg-bg-card rounded-full peer-checked:bg-accent transition-colors"
                ></div>
                <div
                  class="absolute left-1 top-1 w-4 h-4 bg-text-secondary rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-white"
                ></div>
              </div>
            </label>
          </div>
        </fieldset>

        <!-- Export Name -->
        <div>
          <label for="export-name" class="block text-sm font-medium text-text-secondary mb-2">
            Export Name
          </label>
          <input
            id="export-name"
            type="text"
            bind:value={exportName}
            placeholder="Enter export folder name..."
            class="w-full bg-bg-card text-text-primary text-sm border border-bg-card rounded-lg px-3 py-2.5 focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50"
          />
          <p class="text-xs text-text-secondary/70 mt-1.5">
            Folder: <span class="text-text-secondary"
              >{exportName.trim() || currentProject.value?.name || "Project"}</span
            >
          </p>
        </div>

        <!-- Destination Folder -->
        <div>
          <label for="destination" class="block text-sm font-medium text-text-secondary mb-2">
            Destination Folder
          </label>
          <div class="flex gap-2">
            <input
              id="destination"
              type="text"
              readonly
              value={outputPath}
              placeholder="Select a folder..."
              class="flex-1 bg-bg-card text-text-primary text-sm border border-bg-card rounded-lg px-3 py-2.5 focus:outline-none focus:border-accent cursor-pointer truncate"
              onclick={selectDestination}
            />
            <Tooltip text="Browse" position="top">
              <button
                type="button"
                onclick={selectDestination}
                class="px-3 py-2.5 bg-bg-card text-text-secondary rounded-lg hover:bg-beat-header hover:text-text-primary transition-colors border border-bg-card"
                aria-label="Browse for folder"
              >
                <FolderOpen class="w-5 h-5" />
              </button>
            </Tooltip>
          </div>
        </div>
      {/if}

      <!-- Snapshot Option (shown for both formats) -->
      <div class="pt-2 border-t border-bg-card/50">
        <label
          class="flex items-center justify-between p-3 bg-bg-card/30 rounded-lg cursor-pointer hover:bg-bg-card/50 transition-colors"
        >
          <div>
            <span class="text-sm text-text-primary">Create snapshot before exporting</span>
            <p class="text-xs text-text-secondary/70 mt-0.5">Save a backup of your current work</p>
          </div>
          <div class="relative">
            <input type="checkbox" bind:checked={createSnapshot} class="peer sr-only" />
            <div
              class="w-10 h-6 bg-bg-card rounded-full peer-checked:bg-accent transition-colors"
            ></div>
            <div
              class="absolute left-1 top-1 w-4 h-4 bg-text-secondary rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-white"
            ></div>
          </div>
        </label>
      </div>

      <!-- Error Message -->
      {#if error}
        <div class="p-3 bg-red-500/10 border border-red-500/20 rounded-lg">
          <p class="text-sm text-red-400">{error}</p>
        </div>
      {/if}
    </div>

    <!-- Footer -->
    <div
      class="flex items-center justify-end gap-3 px-5 py-4 border-t border-bg-card flex-shrink-0 bg-bg-panel"
    >
      <button
        type="button"
        onclick={onClose}
        class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors rounded-lg hover:bg-bg-card"
        disabled={exporting}
      >
        Cancel
      </button>
      <button
        type="button"
        onclick={handleExport}
        class="px-5 py-2 text-sm font-medium bg-accent text-white rounded-lg hover:bg-accent/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 shadow-lg shadow-accent/20"
        disabled={!canExport || exporting}
      >
        {#if exporting}
          <Loader2 class="w-4 h-4 animate-spin" />
          Exporting...
        {:else}
          <FileDown class="w-4 h-4" />
          Export
        {/if}
      </button>
    </div>
  </div>
</div>
