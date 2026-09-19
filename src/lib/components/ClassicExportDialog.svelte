<!--
  ExportDialog.svelte - Export configuration dialog

  Allows users to configure and initiate project exports:
  - Format selection (Markdown, Longform, Word Document, or ePub)
  - Scope selection based on context (project/chapter/scene)
  - Options like beat markers, synopsis, page breaks
  - Destination folder/file picker
-->
<script lang="ts">
  import { onMount } from "svelte";
  import {
    decodeProfiles,
    profileStorageKey,
    starterProfiles,
    formatLabels,
    type ExportProfile,
  } from "../utils/exportPrototype";
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
    Book,
    Image as ImageIcon,
    ScrollText,
    PenTool,
    Settings2,
  } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import type {
    ExportResult,
    MarkdownExportOptions,
    LongformExportOptions,
    DocxExportOptions,
    EpubExportOptions,
    ExportScope,
    ChapterHeadingStyle,
    SceneBreakStyle,
    FontFamily,
    LineSpacingOption,
    EpubTheme,
    TreatmentLevel,
    TreatmentFormat,
    TreatmentOptions,
    ScrivenerExportMode,
    ScrivenerExportOptions,
    NovelWriterExportOptions,
  } from "../types";
  import ScrivenerMatchDialog from "./ScrivenerMatchDialog.svelte";
  import Tooltip from "./Tooltip.svelte";

  const LAST_EXPORT_PATH_KEY = "kindling:lastExportPath";

  let {
    scope,
    scopeId,
    scopeTitle,
    onClose,
    onSuccess,
    onCustomize,
  }: {
    scope: "project" | "chapter" | "scene";
    scopeId: string | null;
    scopeTitle: string;
    onClose: () => void;
    onSuccess: (result: ExportResult) => void;
    onCustomize: () => void;
  } = $props();

  const exportFormats = [
    { id: "novelwriter", label: "novelWriter", detail: "Project folder", icon: BookOpen },
    { id: "docx", label: "Word", detail: ".docx", icon: FileText },
    { id: "markdown", label: "Markdown", detail: ".md files", icon: BookOpen },
    { id: "longform", label: "Longform", detail: "Index + scenes", icon: AlignLeft },
    { id: "epub", label: "ePub", detail: ".epub", icon: Book },
    { id: "treatment", label: "Treatment", detail: ".docx / .txt", icon: ScrollText },
    { id: "scrivener", label: "Scrivener", detail: ".scriv", icon: PenTool },
    { id: "custom", label: "Custom", detail: "Your saved settings", icon: Settings2 },
  ] as const;

  let exportFormat = $state<
    "markdown" | "longform" | "docx" | "epub" | "treatment" | "scrivener" | "novelwriter" | "custom"
  >("docx");
  let customProfiles = $state<ExportProfile[]>([]);
  let customProfileId = $state("");
  let profileLoadFailed = $state(false);
  const selectedCustomProfile = $derived(customProfiles.find((p) => p.id === customProfileId));
  const customPreferenceKey = () => `kindling:custom-export:${currentProject.value?.id}`;

  onMount(() => {
    const project = currentProject.value;
    if (!project) return;
    try {
      const raw = localStorage.getItem(profileStorageKey(project.id));
      const saved = raw ? decodeProfiles(raw) : null;
      customProfiles =
        saved?.profiles ?? starterProfiles(project.name, project.author_pen_name ?? "");
      customProfileId = saved?.activeId ?? customProfiles[0].id;
      if (localStorage.getItem(customPreferenceKey()) === "true") exportFormat = "custom";
    } catch {
      profileLoadFailed = true;
    }
  });

  function rememberCustomProfile() {
    const project = currentProject.value;
    if (!project || profileLoadFailed) return;
    try {
      const raw = localStorage.getItem(profileStorageKey(project.id));
      // Retain the workspace draft and saved settings; only change the active choice.
      const saved = raw
        ? JSON.parse(raw)
        : { version: 2, profiles: $state.snapshot(customProfiles) };
      localStorage.setItem(
        profileStorageKey(project.id),
        JSON.stringify({ ...saved, activeId: customProfileId })
      );
      localStorage.setItem(customPreferenceKey(), "true");
      error = null;
    } catch (e) {
      error = `Could not remember the selected profile: ${String(e)}`;
    }
  }

  let includeBeatMarkers = $state(false);
  let includeSynopsis = $state(false);
  let pageBreaksBetweenChapters = $state(true);
  let includeTitlePage = $state(true);
  let chapterHeadingStyle = $state<ChapterHeadingStyle>("number_only");
  let sceneBreakStyle = $state<SceneBreakStyle>("hash");
  let fontFamily = $state<FontFamily>("courier_new");
  let lineSpacing = $state<LineSpacingOption>("double");
  let epubTheme = $state<EpubTheme>("classic");
  let epubTitle = $state("");
  let epubAuthor = $state("");
  let epubDescription = $state("");
  let epubLanguage = $state("en");
  let includeCoverImage = $state(false);
  let coverImagePath = $state("");
  let treatmentLevel = $state<TreatmentLevel>("five_page");
  let treatmentFormat = $state<TreatmentFormat>("docx");
  let treatmentFilePath = $state("");
  let novelwriterPath = $state("");
  let novelwriterBeatComments = $state(true);
  let novelwriterNotes = $state(true);
  let scrivenerMode = $state<ScrivenerExportMode>("create_new");
  let scrivenerPath = $state("");
  let scrivenerBackup = $state(true);
  let scrivenerIncludeUnmatched = $state(true);
  let showMatchPreview = $state(false);
  let deleteExisting = $state(false);
  let createSnapshot = $state(false);
  let outputPath = $state("");
  let docxFilePath = $state("");
  let epubFilePath = $state("");
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

  const epubThemeOptions: { value: EpubTheme; label: string }[] = [
    { value: "classic", label: "Classic" },
    { value: "modern", label: "Modern" },
    { value: "minimal", label: "Minimal" },
  ];

  // Initialize export name from project name
  $effect(() => {
    if (currentProject.value && !exportName) {
      exportName = currentProject.value.name;
    }
  });

  // Initialize EPUB metadata defaults
  $effect(() => {
    if (currentProject.value) {
      if (!epubTitle) {
        epubTitle = currentProject.value.name;
      }
      if (!epubAuthor && currentProject.value.author_pen_name) {
        epubAuthor = currentProject.value.author_pen_name;
      }
      if (!epubLanguage) {
        epubLanguage = "en";
      }
    }
  });

  // Load last export path from localStorage on mount
  $effect(() => {
    const savedPath = localStorage.getItem(LAST_EXPORT_PATH_KEY);
    if (savedPath && !outputPath) {
      outputPath = savedPath;
    }
  });

  // Reset treatment file path when treatment output format changes
  $effect(() => {
    void treatmentFormat;
    treatmentFilePath = "";
  });

  // Reset scrivener path when mode changes
  $effect(() => {
    void scrivenerMode;
    scrivenerPath = "";
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
  const formattedWordCount = $derived.by(() => {
    if (wordCount === null) return null;
    if (wordCount < 1000) return `${wordCount} words`;
    const rounded = Math.round(wordCount / 1000) * 1000;
    return `~${rounded.toLocaleString()} words`;
  });

  const canExport = $derived(
    (exportFormat === "custom" && !!selectedCustomProfile && !profileLoadFailed) ||
      (exportFormat === "novelwriter" &&
        novelwriterPath.length > 0 &&
        scope === "project" &&
        currentProject.value?.project_type !== "screenplay") ||
      (exportFormat === "markdown" && outputPath.length > 0) ||
      (exportFormat === "longform" && outputPath.length > 0) ||
      (exportFormat === "docx" && docxFilePath.length > 0) ||
      (exportFormat === "epub" &&
        epubFilePath.length > 0 &&
        (!includeCoverImage || coverImagePath.length > 0)) ||
      (exportFormat === "treatment" && treatmentFilePath.length > 0) ||
      (exportFormat === "scrivener" && scrivenerPath.length > 0)
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

  async function selectEpubFile() {
    const defaultName = `${epubTitle.trim() || currentProject.value?.name || "Export"}.epub`;
    const path = await save({
      title: "Save EPUB",
      defaultPath: defaultName,
      filters: [{ name: "EPUB", extensions: ["epub"] }],
    });

    if (path) {
      epubFilePath = path;
      error = null;
    }
  }

  async function selectCoverImage() {
    const path = await open({
      title: "Select Cover Image",
      filters: [{ name: "Images", extensions: ["jpg", "jpeg", "png", "gif", "webp"] }],
    });

    if (path) {
      coverImagePath = path;
      error = null;
    }
  }

  async function selectTreatmentFile() {
    const ext = treatmentFormat === "docx" ? "docx" : "txt";
    const filterName = treatmentFormat === "docx" ? "Word Document" : "Text File";
    const defaultName = `${currentProject.value?.name || "Treatment"} - Treatment.${ext}`;
    const path = await save({
      title: "Save Treatment",
      defaultPath: defaultName,
      filters: [{ name: filterName, extensions: [ext] }],
    });

    if (path) {
      treatmentFilePath = path;
      error = null;
    }
  }

  async function selectNovelWriterPath() {
    try {
      const path = await open({
        directory: true,
        multiple: false,
        title: "Choose an empty novelWriter destination folder",
      });
      if (path) {
        novelwriterPath = path;
        error = null;
      }
    } catch (e) {
      error = String(e);
    }
  }

  async function selectScrivenerPath() {
    if (scrivenerMode === "create_new") {
      const path = await save({
        title: "Save Scrivener Project",
        defaultPath: `${currentProject.value?.name || "Export"}.scriv`,
        filters: [{ name: "Scrivener Project", extensions: ["scriv"] }],
      });
      if (path) {
        scrivenerPath = path;
        error = null;
      }
    } else {
      const path = await open({
        directory: true,
        title: "Select Existing .scriv Bundle",
        defaultPath: currentProject.value?.source_path || undefined,
      });
      if (path) {
        scrivenerPath = path;
        error = null;
      }
    }
  }

  async function handleExport() {
    if (exportFormat === "custom") {
      rememberCustomProfile();
      if (!error) onCustomize();
      return;
    }
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
      } else if (exportFormat === "longform") {
        const options: LongformExportOptions = {
          scope: exportScope,
          output_path: outputPath,
          export_name: exportName.trim() || undefined,
          delete_existing: deleteExisting,
          create_snapshot: createSnapshot,
        };

        result = await invoke<ExportResult>("export_to_longform", {
          projectId: currentProject.value.id,
          options,
        });

        localStorage.setItem(LAST_EXPORT_PATH_KEY, outputPath);
      } else if (exportFormat === "docx") {
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
      } else if (exportFormat === "treatment") {
        const options: TreatmentOptions = {
          detail_level: treatmentLevel,
          format: treatmentFormat,
          output_path: treatmentFilePath,
          create_snapshot: createSnapshot,
        };

        result = await invoke<ExportResult>("generate_treatment", {
          projectId: currentProject.value.id,
          options,
        });
      } else if (exportFormat === "novelwriter") {
        const options: NovelWriterExportOptions = {
          include_beat_comments: novelwriterBeatComments,
          include_notes: novelwriterNotes,
          create_snapshot: createSnapshot,
        };
        result = await invoke<ExportResult>("export_to_novelwriter", {
          projectId: currentProject.value.id,
          outputPath: novelwriterPath,
          options,
        });
      } else if (exportFormat === "scrivener") {
        if (scrivenerMode === "update" && !showMatchPreview) {
          showMatchPreview = true;
          exporting = false;
          return;
        }
        showMatchPreview = false;

        const options: ScrivenerExportOptions = {
          mode: scrivenerMode,
          output_path: scrivenerPath,
          backup: scrivenerBackup,
          include_unmatched: scrivenerIncludeUnmatched,
          create_snapshot: createSnapshot,
        };

        result = await invoke<ExportResult>("export_to_scrivener", {
          projectId: currentProject.value.id,
          options,
        });
      } else {
        const options: EpubExportOptions = {
          scope: exportScope,
          include_beat_markers: includeBeatMarkers,
          include_synopsis: includeSynopsis,
          output_path: epubFilePath,
          create_snapshot: createSnapshot,
          metadata: {
            title: epubTitle.trim(),
            author: epubAuthor.trim(),
            description: epubDescription.trim() || undefined,
            language: epubLanguage.trim() || "en",
          },
          theme: epubTheme,
          include_cover_image: includeCoverImage,
          cover_image_path: includeCoverImage ? coverImagePath.trim() : undefined,
        };

        result = await invoke<ExportResult>("export_to_epub", {
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
  class="fixed inset-0 z-press-modal flex items-center justify-center bg-press-overlay"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="export-dialog-title"
  tabindex="-1"
>
  <!-- Dialog -->
  <div
    class="app-dialog-surface bg-press-surface rounded-lg shadow-press-overlay w-full max-w-lg mx-4 overflow-hidden max-h-[90vh] flex flex-col"
  >
    <!-- Header -->
    <div
      class="flex items-center justify-between px-5 py-4 border-b border-press-border flex-shrink-0"
    >
      <div class="flex items-center gap-3">
        <div class="p-2 bg-press-accent-wash rounded-lg">
          <FileDown class="w-5 h-5 text-press-accent-text" />
        </div>
        <div>
          <h2 id="export-dialog-title" class="text-press-body-lg font-medium text-press-text">
            Export {scopeTitle}
          </h2>
          <p class="text-press-eyebrow text-press-muted">Choose format and configure options</p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        {#if scope === "project"}
          <div class="flex items-center gap-1.5 px-2.5 py-1.5 bg-press-sunken rounded-lg">
            <Hash class="w-3.5 h-3.5 text-press-muted" />
            {#if loadingWordCount}
              <span class="text-press-eyebrow text-press-muted">...</span>
            {:else if formattedWordCount}
              <span class="text-press-eyebrow text-press-muted">{formattedWordCount}</span>
            {/if}
          </div>
        {/if}
        <Tooltip text="Close" position="left">
          <button
            type="button"
            onclick={onClose}
            class="p-2 text-press-muted hover:text-press-text hover:bg-press-sunken transition-colors rounded-lg"
            aria-label="Close"
            data-testid="export-close"
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
        <legend class="block text-press-ui font-medium text-press-muted mb-3">Export Format</legend>
        <div class="grid grid-cols-2 sm:grid-cols-3 auto-rows-fr gap-3">
          {#each exportFormats as format (format.id)}
            {#if format.id !== "novelwriter" || (scope === "project" && currentProject.value?.project_type !== "screenplay")}
              <label
                class="relative flex flex-col items-center justify-center text-center p-4 rounded-lg border-2 cursor-pointer transition-all {exportFormat ===
                format.id
                  ? 'border-press-accent bg-press-accent-wash'
                  : 'border-press-border bg-press-sunken'}"
              >
                <input
                  type="radio"
                  name="format"
                  data-testid={`export-format-${format.id}`}
                  value={format.id}
                  bind:group={exportFormat}
                  onchange={() => {
                    if (format.id === "custom") rememberCustomProfile();
                  }}
                  class="sr-only"
                />
                <format.icon
                  class="w-8 h-8 mb-2 {exportFormat === format.id
                    ? 'text-press-accent-text'
                    : 'text-press-muted'}"
                />
                <span
                  class="text-press-ui font-medium {exportFormat === format.id
                    ? 'text-press-text'
                    : 'text-press-muted'}">{format.label}</span
                >
                <span class="text-press-eyebrow text-press-muted mt-0.5">{format.detail}</span>
                {#if exportFormat === format.id}
                  <div class="absolute top-2 right-2 w-2 h-2 rounded-full bg-press-accent"></div>
                {/if}
              </label>
            {/if}
          {/each}
        </div>
      </fieldset>

      {#if exportFormat === "custom"}
        <div class="space-y-3">
          <label for="custom-export-profile" class="block text-press-ui font-medium text-press-text"
            >Export profile</label
          >
          <select
            id="custom-export-profile"
            bind:value={customProfileId}
            onchange={(event) => {
              customProfileId = event.currentTarget.value;
              rememberCustomProfile();
            }}
            disabled={profileLoadFailed}
            class="w-full px-3 py-2.5 bg-press-surface text-press-text border border-press-border rounded-lg text-press-ui"
          >
            {#each customProfiles as profile}<option value={profile.id}>{profile.name}</option
              >{/each}
          </select>
          {#if profileLoadFailed}<p class="text-press-ui text-press-error" role="alert">
              Saved profiles could not be loaded. Close and reopen Export to try again.
            </p>
          {:else}<p class="text-press-ui text-press-muted">
              {selectedCustomProfile
                ? formatLabels[selectedCustomProfile.format] + ". "
                : ""}Preview, adjust, and export in the workspace. This project remembers your
              chosen profile.
            </p>{/if}
        </div>
      {/if}

      {#if exportFormat === "docx"}
        <!-- DOCX Options Section -->
        <fieldset>
          <legend
            class="flex items-center gap-2 text-press-ui font-medium text-press-accent-text mb-3"
          >
            <Type class="w-4 h-4" />
            Document Structure
          </legend>

          <!-- Toggle Options -->
          <div class="space-y-2 mb-4">
            <label
              class="flex items-center justify-between p-3 bg-press-sunken rounded-lg cursor-pointer hover:bg-press-sunken transition-colors group"
            >
              <div class="flex items-center gap-3">
                <span class="text-press-ui text-press-text">Include title page</span>
              </div>
              <div class="relative">
                <input type="checkbox" bind:checked={includeTitlePage} class="peer sr-only" />
                <div
                  class="w-10 h-6 bg-press-sunken rounded-full peer-checked:bg-press-accent transition-colors"
                ></div>
                <div
                  class="absolute left-1 top-1 w-4 h-4 bg-press-muted rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-press-on-accent"
                ></div>
              </div>
            </label>

            <label
              class="flex items-center justify-between p-3 bg-press-sunken rounded-lg cursor-pointer hover:bg-press-sunken transition-colors"
            >
              <span class="text-press-ui text-press-text">Page breaks between chapters</span>
              <div class="relative">
                <input
                  type="checkbox"
                  bind:checked={pageBreaksBetweenChapters}
                  class="peer sr-only"
                />
                <div
                  class="w-10 h-6 bg-press-sunken rounded-full peer-checked:bg-press-accent transition-colors"
                ></div>
                <div
                  class="absolute left-1 top-1 w-4 h-4 bg-press-muted rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-press-on-accent"
                ></div>
              </div>
            </label>

            <label
              class="flex items-center justify-between p-3 bg-press-sunken rounded-lg cursor-pointer hover:bg-press-sunken transition-colors"
            >
              <span class="text-press-ui text-press-text">Include beat markers as headings</span>
              <div class="relative">
                <input type="checkbox" bind:checked={includeBeatMarkers} class="peer sr-only" />
                <div
                  class="w-10 h-6 bg-press-sunken rounded-full peer-checked:bg-press-accent transition-colors"
                ></div>
                <div
                  class="absolute left-1 top-1 w-4 h-4 bg-press-muted rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-press-on-accent"
                ></div>
              </div>
            </label>

            <label
              class="flex items-center justify-between p-3 bg-press-sunken rounded-lg cursor-pointer hover:bg-press-sunken transition-colors"
            >
              <span class="text-press-ui text-press-text">Include scene synopses</span>
              <div class="relative">
                <input type="checkbox" bind:checked={includeSynopsis} class="peer sr-only" />
                <div
                  class="w-10 h-6 bg-press-sunken rounded-full peer-checked:bg-press-accent transition-colors"
                ></div>
                <div
                  class="absolute left-1 top-1 w-4 h-4 bg-press-muted rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-press-on-accent"
                ></div>
              </div>
            </label>
          </div>

          <!-- Dropdown Selects -->
          <div class="grid grid-cols-2 gap-3">
            <!-- Chapter Heading Style -->
            <div>
              <label
                for="chapter-heading-style"
                class="block text-press-eyebrow text-press-muted mb-1.5"
              >
                Chapter Heading
              </label>
              <div class="relative">
                <select
                  id="chapter-heading-style"
                  bind:value={chapterHeadingStyle}
                  class="w-full appearance-none bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg pl-3 pr-8 py-2.5 focus:outline-none focus:border-press-accent focus:ring-1 focus:ring-press-focus cursor-pointer"
                >
                  {#each chapterHeadingStyles as style (style.value)}
                    <option value={style.value}>{style.label}</option>
                  {/each}
                </select>
                <ChevronDown
                  class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-press-muted pointer-events-none"
                />
              </div>
              <p class="text-press-eyebrow text-press-muted mt-1 truncate">
                {chapterHeadingStyles.find((s) => s.value === chapterHeadingStyle)?.example}
              </p>
            </div>

            <!-- Scene Break Style -->
            <div>
              <label
                for="scene-break-style"
                class="block text-press-eyebrow text-press-muted mb-1.5"
              >
                Scene Break
              </label>
              <div class="relative">
                <select
                  id="scene-break-style"
                  bind:value={sceneBreakStyle}
                  class="w-full appearance-none bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg pl-3 pr-8 py-2.5 focus:outline-none focus:border-press-accent focus:ring-1 focus:ring-press-focus cursor-pointer"
                >
                  {#each sceneBreakStyles as style (style.value)}
                    <option value={style.value}>{style.label}</option>
                  {/each}
                </select>
                <ChevronDown
                  class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-press-muted pointer-events-none"
                />
              </div>
              <p class="text-press-eyebrow text-press-muted mt-1">
                {sceneBreakStyles.find((s) => s.value === sceneBreakStyle)?.example}
              </p>
            </div>
          </div>
        </fieldset>

        <!-- Typography Section -->
        <fieldset>
          <legend
            class="flex items-center gap-2 text-press-ui font-medium text-press-accent-text mb-3"
          >
            <AlignLeft class="w-4 h-4" />
            Typography
          </legend>
          <div class="grid grid-cols-2 gap-3">
            <!-- Font Family -->
            <div>
              <label for="font-family" class="block text-press-eyebrow text-press-muted mb-1.5">
                Font
              </label>
              <div class="relative">
                <select
                  id="font-family"
                  bind:value={fontFamily}
                  class="w-full appearance-none bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg pl-3 pr-8 py-2.5 focus:outline-none focus:border-press-accent focus:ring-1 focus:ring-press-focus cursor-pointer"
                >
                  {#each fontFamilies as font (font.value)}
                    <option value={font.value}>{font.label}</option>
                  {/each}
                </select>
                <ChevronDown
                  class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-press-muted pointer-events-none"
                />
              </div>
            </div>

            <!-- Line Spacing -->
            <div>
              <label for="line-spacing" class="block text-press-eyebrow text-press-muted mb-1.5">
                Line Spacing
              </label>
              <div class="relative">
                <select
                  id="line-spacing"
                  bind:value={lineSpacing}
                  class="w-full appearance-none bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg pl-3 pr-8 py-2.5 focus:outline-none focus:border-press-accent focus:ring-1 focus:ring-press-focus cursor-pointer"
                >
                  {#each lineSpacingOptions as spacing (spacing.value)}
                    <option value={spacing.value}>{spacing.label}</option>
                  {/each}
                </select>
                <ChevronDown
                  class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-press-muted pointer-events-none"
                />
              </div>
            </div>
          </div>
        </fieldset>

        <!-- Save Location -->
        <div>
          <label
            for="docx-destination"
            class="block text-press-ui font-medium text-press-muted mb-2"
          >
            Save Location
          </label>
          <div class="flex gap-2">
            <input
              id="docx-destination"
              type="text"
              readonly
              value={docxFilePath}
              placeholder="Choose where to save..."
              class="flex-1 bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg px-3 py-2.5 focus:outline-none focus:border-press-accent cursor-pointer truncate"
              onclick={selectDocxFile}
            />
            <Tooltip text="Browse" position="top">
              <button
                type="button"
                onclick={selectDocxFile}
                class="px-3 py-2.5 bg-press-sunken text-press-muted rounded-lg hover:bg-press-sunken hover:text-press-text transition-colors border border-press-border"
                aria-label="Choose save location"
              >
                <FileText class="w-5 h-5" />
              </button>
            </Tooltip>
          </div>
        </div>
      {:else if exportFormat === "markdown"}
        <!-- Markdown Options -->
        <fieldset>
          <legend
            class="flex items-center gap-2 text-press-ui font-medium text-press-accent-text mb-3"
          >
            <Type class="w-4 h-4" />
            Options
          </legend>

          <div class="space-y-2 mb-4">
            <label
              class="flex items-center justify-between p-3 bg-press-sunken rounded-lg cursor-pointer hover:bg-press-sunken transition-colors"
            >
              <span class="text-press-ui text-press-text">Include beat markers as headings</span>
              <div class="relative">
                <input type="checkbox" bind:checked={includeBeatMarkers} class="peer sr-only" />
                <div
                  class="w-10 h-6 bg-press-sunken rounded-full peer-checked:bg-press-accent transition-colors"
                ></div>
                <div
                  class="absolute left-1 top-1 w-4 h-4 bg-press-muted rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-press-on-accent"
                ></div>
              </div>
            </label>

            <label
              class="flex items-center justify-between p-3 bg-press-sunken rounded-lg cursor-pointer hover:bg-press-sunken transition-colors"
            >
              <span class="text-press-ui text-press-text">Delete existing export folder</span>
              <div class="relative">
                <input type="checkbox" bind:checked={deleteExisting} class="peer sr-only" />
                <div
                  class="w-10 h-6 bg-press-sunken rounded-full peer-checked:bg-press-accent transition-colors"
                ></div>
                <div
                  class="absolute left-1 top-1 w-4 h-4 bg-press-muted rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-press-on-accent"
                ></div>
              </div>
            </label>
          </div>
        </fieldset>

        <!-- Export Name -->
        <div>
          <label for="export-name" class="block text-press-ui font-medium text-press-muted mb-2">
            Export Name
          </label>
          <input
            id="export-name"
            type="text"
            bind:value={exportName}
            placeholder="Enter export folder name..."
            class="w-full bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg px-3 py-2.5 focus:outline-none focus:border-press-accent focus:ring-1 focus:ring-press-focus"
          />
          <p class="text-press-eyebrow text-press-muted mt-1.5">
            Folder: <span class="text-press-muted"
              >{exportName.trim() || currentProject.value?.name || "Project"}</span
            >
          </p>
        </div>

        <!-- Destination Folder -->
        <div>
          <label for="destination" class="block text-press-ui font-medium text-press-muted mb-2">
            Destination Folder
          </label>
          <div class="flex gap-2">
            <input
              id="destination"
              type="text"
              readonly
              value={outputPath}
              placeholder="Select a folder..."
              class="flex-1 bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg px-3 py-2.5 focus:outline-none focus:border-press-accent cursor-pointer truncate"
              onclick={selectDestination}
            />
            <Tooltip text="Browse" position="top">
              <button
                type="button"
                onclick={selectDestination}
                class="px-3 py-2.5 bg-press-sunken text-press-muted rounded-lg hover:bg-press-sunken hover:text-press-text transition-colors border border-press-border"
                aria-label="Browse for folder"
              >
                <FolderOpen class="w-5 h-5" />
              </button>
            </Tooltip>
          </div>
        </div>
      {:else if exportFormat === "longform"}
        <!-- Longform Options -->
        <fieldset>
          <legend
            class="flex items-center gap-2 text-press-ui font-medium text-press-accent-text mb-3"
          >
            <Type class="w-4 h-4" />
            Options
          </legend>

          <div class="space-y-2 mb-4">
            <label
              class="flex items-center justify-between p-3 bg-press-sunken rounded-lg cursor-pointer hover:bg-press-sunken transition-colors"
            >
              <span class="text-press-ui text-press-text">Delete existing export folder</span>
              <div class="relative">
                <input type="checkbox" bind:checked={deleteExisting} class="peer sr-only" />
                <div
                  class="w-10 h-6 bg-press-sunken rounded-full peer-checked:bg-press-accent transition-colors"
                ></div>
                <div
                  class="absolute left-1 top-1 w-4 h-4 bg-press-muted rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-press-on-accent"
                ></div>
              </div>
            </label>
          </div>
        </fieldset>

        <!-- Export Name -->
        <div>
          <label
            for="export-name-longform"
            class="block text-press-ui font-medium text-press-muted mb-2"
          >
            Export Name
          </label>
          <input
            id="export-name-longform"
            type="text"
            bind:value={exportName}
            placeholder="Enter project name..."
            class="w-full bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg px-3 py-2.5 focus:outline-none focus:border-press-accent focus:ring-1 focus:ring-press-focus"
          />
          <p class="text-press-eyebrow text-press-muted mt-1.5">
            Folder: <span class="text-press-muted"
              >{exportName.trim() || currentProject.value?.name || "Project"}</span
            >
            · Index:
            <span class="text-press-muted"
              >{exportName.trim() || currentProject.value?.name || "Project"}.md</span
            >
          </p>
        </div>

        <!-- Destination Folder -->
        <div>
          <label
            for="destination-longform"
            class="block text-press-ui font-medium text-press-muted mb-2"
          >
            Destination Folder
          </label>
          <div class="flex gap-2">
            <input
              id="destination-longform"
              type="text"
              readonly
              value={outputPath}
              placeholder="Select a folder..."
              class="flex-1 bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg px-3 py-2.5 focus:outline-none focus:border-press-accent cursor-pointer truncate"
              onclick={selectDestination}
            />
            <Tooltip text="Browse" position="top">
              <button
                type="button"
                onclick={selectDestination}
                class="px-3 py-2.5 bg-press-sunken text-press-muted rounded-lg hover:bg-press-sunken hover:text-press-text transition-colors border border-press-border"
                aria-label="Browse for folder"
              >
                <FolderOpen class="w-5 h-5" />
              </button>
            </Tooltip>
          </div>
        </div>
      {:else if exportFormat === "treatment"}
        <!-- Treatment Options -->
        <fieldset>
          <legend
            class="flex items-center gap-2 text-press-ui font-medium text-press-accent-text mb-3"
          >
            <ScrollText class="w-4 h-4" />
            Detail Level
          </legend>
          <div class="space-y-2">
            <label
              class="flex items-center justify-between p-3 rounded-lg cursor-pointer transition-all {treatmentLevel ===
              'one_page'
                ? 'bg-press-accent-wash border border-press-accent'
                : 'bg-press-sunken hover:bg-press-sunken border border-transparent'}"
            >
              <div>
                <span class="text-press-ui text-press-text">One-Page</span>
                <p class="text-press-eyebrow text-press-muted mt-0.5">
                  Title, logline, and a short synopsis per act
                </p>
              </div>
              <input
                type="radio"
                name="treatment-level"
                value="one_page"
                bind:group={treatmentLevel}
                class="accent-accent"
              />
            </label>
            <label
              class="flex items-center justify-between p-3 rounded-lg cursor-pointer transition-all {treatmentLevel ===
              'five_page'
                ? 'bg-press-accent-wash border border-press-accent'
                : 'bg-press-sunken hover:bg-press-sunken border border-transparent'}"
            >
              <div>
                <span class="text-press-ui text-press-text">Five-Page</span>
                <p class="text-press-eyebrow text-press-muted mt-0.5">
                  Act summaries with key scene descriptions
                </p>
              </div>
              <input
                type="radio"
                name="treatment-level"
                value="five_page"
                bind:group={treatmentLevel}
                class="accent-accent"
              />
            </label>
            <label
              class="flex items-center justify-between p-3 rounded-lg cursor-pointer transition-all {treatmentLevel ===
              'full'
                ? 'bg-press-accent-wash border border-press-accent'
                : 'bg-press-sunken hover:bg-press-sunken border border-transparent'}"
            >
              <div>
                <span class="text-press-ui text-press-text">Full Treatment</span>
                <p class="text-press-eyebrow text-press-muted mt-0.5">
                  Every scene synopsis and beat description
                </p>
              </div>
              <input
                type="radio"
                name="treatment-level"
                value="full"
                bind:group={treatmentLevel}
                class="accent-accent"
              />
            </label>
          </div>
        </fieldset>

        <!-- Output Format -->
        <fieldset>
          <legend
            class="flex items-center gap-2 text-press-ui font-medium text-press-accent-text mb-3"
          >
            <FileText class="w-4 h-4" />
            Output Format
          </legend>
          <div class="grid grid-cols-2 gap-3">
            <label
              class="flex items-center justify-center gap-2 p-3 rounded-lg border-2 cursor-pointer transition-all {treatmentFormat ===
              'docx'
                ? 'border-press-accent bg-press-accent-wash'
                : 'border-press-border border-press-border bg-press-sunken'}"
            >
              <input
                type="radio"
                name="treatment-format"
                value="docx"
                bind:group={treatmentFormat}
                class="sr-only"
              />
              <FileText
                class="w-4 h-4 {treatmentFormat === 'docx'
                  ? 'text-press-accent-text'
                  : 'text-press-muted'}"
              />
              <span
                class="text-press-ui font-medium {treatmentFormat === 'docx'
                  ? 'text-press-text'
                  : 'text-press-muted'}">.docx</span
              >
            </label>
            <label
              class="flex items-center justify-center gap-2 p-3 rounded-lg border-2 cursor-pointer transition-all {treatmentFormat ===
              'txt'
                ? 'border-press-accent bg-press-accent-wash'
                : 'border-press-border border-press-border bg-press-sunken'}"
            >
              <input
                type="radio"
                name="treatment-format"
                value="txt"
                bind:group={treatmentFormat}
                class="sr-only"
              />
              <AlignLeft
                class="w-4 h-4 {treatmentFormat === 'txt'
                  ? 'text-press-accent-text'
                  : 'text-press-muted'}"
              />
              <span
                class="text-press-ui font-medium {treatmentFormat === 'txt'
                  ? 'text-press-text'
                  : 'text-press-muted'}">.txt</span
              >
            </label>
          </div>
        </fieldset>

        <!-- Save Location -->
        <div>
          <label
            for="treatment-destination"
            class="block text-press-ui font-medium text-press-muted mb-2"
          >
            Save Location
          </label>
          <div class="flex gap-2">
            <input
              id="treatment-destination"
              type="text"
              readonly
              value={treatmentFilePath}
              placeholder="Choose where to save..."
              class="flex-1 bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg px-3 py-2.5 focus:outline-none focus:border-press-accent cursor-pointer truncate"
              onclick={selectTreatmentFile}
            />
            <Tooltip text="Browse" position="top">
              <button
                type="button"
                onclick={selectTreatmentFile}
                class="px-3 py-2.5 bg-press-sunken text-press-muted rounded-lg hover:bg-press-sunken hover:text-press-text transition-colors border border-press-border"
                aria-label="Choose save location"
              >
                <FileText class="w-5 h-5" />
              </button>
            </Tooltip>
          </div>
        </div>
      {:else if exportFormat === "novelwriter"}
        <div class="space-y-4">
          <p class="text-press-small text-press-muted">
            Export a complete project for novelWriter 26.2 or newer. Choose an empty folder.
          </p>
          <label class="flex items-center gap-3 text-press-ui text-press-text"
            ><input
              type="checkbox"
              data-testid="novelwriter-beat-comments"
              bind:checked={novelwriterBeatComments}
            /> Include beat comments</label
          >
          <p class="text-press-small text-press-muted">
            Turning off beat comments disables beat-level sync. Page-mode prose syncs as a whole
            scene.
          </p>
          <label class="flex items-center gap-3 text-press-ui text-press-text"
            ><input
              type="checkbox"
              data-testid="novelwriter-notes"
              bind:checked={novelwriterNotes}
            /> Include characters, locations and notes</label
          >
          <label for="novelwriter-destination" class="block text-press-ui text-press-muted"
            >Destination folder</label
          >
          <div class="flex gap-2">
            <input
              id="novelwriter-destination"
              readonly
              value={novelwriterPath}
              placeholder="Choose an empty folder"
              class="flex-1 min-w-0 px-3 py-2 bg-press-sunken border border-press-border rounded-lg text-press-base text-press-text"
            />
            <button
              type="button"
              onclick={selectNovelWriterPath}
              aria-label="Choose novelWriter destination folder"
              class="p-2 border border-press-border rounded-lg"
              ><FolderOpen class="w-5 h-5" /></button
            >
          </div>
          <details class="text-press-small text-press-muted">
            <summary>Round-trip limitations</summary>
            <p>
              Export omits underline, planning status, scene type, tags, discovery notes, snapshots
              and custom fields. Scene status is limited to Draft, Revised and Final.
            </p>
            <p>
              Import flattens H4 sections and does not preserve shortcodes, footnotes, alignment,
              indent codes, importance, ignored text, templates, additional novel roots or POV,
              focus, mention and story references.
            </p>
            <p>
              Sync covers chapters, scenes, beats and prose. Notes, reference links and project
              metadata are not synced. Existing source connections are preserved when exporting.
            </p>
          </details>
        </div>
      {:else if exportFormat === "scrivener"}
        <!-- Scrivener Export Options -->
        <fieldset>
          <legend
            class="flex items-center gap-2 text-press-ui font-medium text-press-accent-text mb-3"
          >
            <PenTool class="w-4 h-4" />
            Export Mode
          </legend>
          <div class="space-y-2">
            <label
              class="flex items-center justify-between p-3 rounded-lg cursor-pointer transition-all {scrivenerMode ===
              'create_new'
                ? 'bg-press-accent-wash border border-press-accent'
                : 'bg-press-sunken hover:bg-press-sunken border border-transparent'}"
            >
              <div>
                <span class="text-press-ui text-press-text">Create New</span>
                <p class="text-press-eyebrow text-press-muted mt-0.5">
                  Build a fresh .scriv project from your outline
                </p>
              </div>
              <input
                type="radio"
                name="scrivener-mode"
                value="create_new"
                bind:group={scrivenerMode}
                class="accent-accent"
              />
            </label>
            <label
              class="flex items-center justify-between p-3 rounded-lg cursor-pointer transition-all {scrivenerMode ===
              'update'
                ? 'bg-press-accent-wash border border-press-accent'
                : 'bg-press-sunken hover:bg-press-sunken border border-transparent'}"
            >
              <div>
                <span class="text-press-ui text-press-text">Update Existing</span>
                <p class="text-press-eyebrow text-press-muted mt-0.5">
                  Write prose back into an existing .scriv bundle
                </p>
              </div>
              <input
                type="radio"
                name="scrivener-mode"
                value="update"
                bind:group={scrivenerMode}
                class="accent-accent"
              />
            </label>
          </div>
        </fieldset>

        {#if scrivenerMode === "update"}
          <!-- Update-mode-specific options -->
          <fieldset>
            <legend
              class="flex items-center gap-2 text-press-ui font-medium text-press-accent-text mb-3"
            >
              <Type class="w-4 h-4" />
              Update Options
            </legend>
            <div class="space-y-2">
              <label
                class="flex items-center justify-between p-3 bg-press-sunken rounded-lg cursor-pointer hover:bg-press-sunken transition-colors"
              >
                <div>
                  <span class="text-press-ui text-press-text">Backup before updating</span>
                  <p class="text-press-eyebrow text-press-muted mt-0.5">
                    Creates a timestamped copy of the .scriv bundle
                  </p>
                </div>
                <div class="relative">
                  <input type="checkbox" bind:checked={scrivenerBackup} class="peer sr-only" />
                  <div
                    class="w-10 h-6 bg-press-sunken rounded-full peer-checked:bg-press-accent transition-colors"
                  ></div>
                  <div
                    class="absolute left-1 top-1 w-4 h-4 bg-press-muted rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-press-on-accent"
                  ></div>
                </div>
              </label>

              <label
                class="flex items-center justify-between p-3 bg-press-sunken rounded-lg cursor-pointer hover:bg-press-sunken transition-colors"
              >
                <div>
                  <span class="text-press-ui text-press-text">Include unmatched scenes</span>
                  <p class="text-press-eyebrow text-press-muted mt-0.5">
                    Create new Scrivener documents for scenes without matches
                  </p>
                </div>
                <div class="relative">
                  <input
                    type="checkbox"
                    bind:checked={scrivenerIncludeUnmatched}
                    class="peer sr-only"
                  />
                  <div
                    class="w-10 h-6 bg-press-sunken rounded-full peer-checked:bg-press-accent transition-colors"
                  ></div>
                  <div
                    class="absolute left-1 top-1 w-4 h-4 bg-press-muted rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-press-on-accent"
                  ></div>
                </div>
              </label>
            </div>
          </fieldset>
        {/if}

        <p class="text-press-eyebrow text-press-muted bg-press-sunken rounded-lg px-3 py-2">
          Note: Characters, locations, and scene references are not included in Scrivener exports.
        </p>

        <!-- Save/Select Location -->
        <div>
          <label
            for="scrivener-destination"
            class="block text-press-ui font-medium text-press-muted mb-2"
          >
            {scrivenerMode === "create_new" ? "Save Location" : "Select .scriv Bundle"}
          </label>
          <div class="flex gap-2">
            <input
              id="scrivener-destination"
              type="text"
              readonly
              value={scrivenerPath}
              placeholder={scrivenerMode === "create_new"
                ? "Choose where to save..."
                : "Select existing .scriv folder..."}
              class="flex-1 bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg px-3 py-2.5 focus:outline-none focus:border-press-accent cursor-pointer truncate"
              onclick={selectScrivenerPath}
            />
            <Tooltip text="Browse" position="top">
              <button
                type="button"
                onclick={selectScrivenerPath}
                class="px-3 py-2.5 bg-press-sunken text-press-muted rounded-lg hover:bg-press-sunken hover:text-press-text transition-colors border border-press-border"
                aria-label={scrivenerMode === "create_new"
                  ? "Choose save location"
                  : "Select .scriv bundle"}
              >
                <FolderOpen class="w-5 h-5" />
              </button>
            </Tooltip>
          </div>
        </div>
      {:else if exportFormat === "epub"}
        <!-- EPUB Options -->
        <fieldset>
          <legend
            class="flex items-center gap-2 text-press-ui font-medium text-press-accent-text mb-3"
          >
            <Type class="w-4 h-4" />
            Metadata
          </legend>
          <div class="space-y-3">
            <div>
              <label for="epub-title" class="block text-press-eyebrow text-press-muted mb-1.5"
                >Title</label
              >
              <input
                id="epub-title"
                type="text"
                bind:value={epubTitle}
                placeholder="Book title"
                class="w-full bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg px-3 py-2.5 focus:outline-none focus:border-press-accent focus:ring-1 focus:ring-press-focus"
              />
            </div>
            <div>
              <label for="epub-author" class="block text-press-eyebrow text-press-muted mb-1.5"
                >Author</label
              >
              <input
                id="epub-author"
                type="text"
                bind:value={epubAuthor}
                placeholder="Author name"
                class="w-full bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg px-3 py-2.5 focus:outline-none focus:border-press-accent focus:ring-1 focus:ring-press-focus"
              />
            </div>
            <div>
              <label for="epub-description" class="block text-press-eyebrow text-press-muted mb-1.5"
                >Description</label
              >
              <textarea
                id="epub-description"
                rows="3"
                bind:value={epubDescription}
                placeholder="Short blurb or summary"
                class="w-full bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg px-3 py-2.5 focus:outline-none focus:border-press-accent focus:ring-1 focus:ring-press-focus resize-none"
              ></textarea>
            </div>
            <div>
              <label for="epub-language" class="block text-press-eyebrow text-press-muted mb-1.5"
                >Language</label
              >
              <input
                id="epub-language"
                type="text"
                bind:value={epubLanguage}
                placeholder="en"
                class="w-full bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg px-3 py-2.5 focus:outline-none focus:border-press-accent focus:ring-1 focus:ring-press-focus"
              />
              <p class="text-press-eyebrow text-press-muted mt-1">
                Use ISO 639-1 codes (e.g., en, es).
              </p>
            </div>
          </div>
        </fieldset>

        <fieldset>
          <legend
            class="flex items-center gap-2 text-press-ui font-medium text-press-accent-text mb-3"
          >
            <AlignLeft class="w-4 h-4" />
            Content
          </legend>
          <div class="space-y-2">
            <label
              class="flex items-center justify-between p-3 bg-press-sunken rounded-lg cursor-pointer hover:bg-press-sunken transition-colors"
            >
              <span class="text-press-ui text-press-text">Include beat markers as headings</span>
              <div class="relative">
                <input type="checkbox" bind:checked={includeBeatMarkers} class="peer sr-only" />
                <div
                  class="w-10 h-6 bg-press-sunken rounded-full peer-checked:bg-press-accent transition-colors"
                ></div>
                <div
                  class="absolute left-1 top-1 w-4 h-4 bg-press-muted rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-press-on-accent"
                ></div>
              </div>
            </label>
            <label
              class="flex items-center justify-between p-3 bg-press-sunken rounded-lg cursor-pointer hover:bg-press-sunken transition-colors"
            >
              <span class="text-press-ui text-press-text">Include scene synopses</span>
              <div class="relative">
                <input type="checkbox" bind:checked={includeSynopsis} class="peer sr-only" />
                <div
                  class="w-10 h-6 bg-press-sunken rounded-full peer-checked:bg-press-accent transition-colors"
                ></div>
                <div
                  class="absolute left-1 top-1 w-4 h-4 bg-press-muted rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-press-on-accent"
                ></div>
              </div>
            </label>
          </div>
        </fieldset>

        <fieldset>
          <legend
            class="flex items-center gap-2 text-press-ui font-medium text-press-accent-text mb-3"
          >
            <Type class="w-4 h-4" />
            Styling
          </legend>
          <div>
            <label for="epub-theme" class="block text-press-eyebrow text-press-muted mb-1.5"
              >Theme</label
            >
            <div class="relative">
              <select
                id="epub-theme"
                bind:value={epubTheme}
                class="w-full appearance-none bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg pl-3 pr-8 py-2.5 focus:outline-none focus:border-press-accent focus:ring-1 focus:ring-press-focus cursor-pointer"
              >
                {#each epubThemeOptions as theme (theme.value)}
                  <option value={theme.value}>{theme.label}</option>
                {/each}
              </select>
              <ChevronDown
                class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-press-muted pointer-events-none"
              />
            </div>
          </div>
        </fieldset>

        <fieldset>
          <legend
            class="flex items-center gap-2 text-press-ui font-medium text-press-accent-text mb-3"
          >
            <ImageIcon class="w-4 h-4" />
            Cover
          </legend>
          <div class="space-y-3">
            <label
              class="flex items-center justify-between p-3 bg-press-sunken rounded-lg cursor-pointer hover:bg-press-sunken transition-colors"
            >
              <span class="text-press-ui text-press-text">Include cover image</span>
              <div class="relative">
                <input type="checkbox" bind:checked={includeCoverImage} class="peer sr-only" />
                <div
                  class="w-10 h-6 bg-press-sunken rounded-full peer-checked:bg-press-accent transition-colors"
                ></div>
                <div
                  class="absolute left-1 top-1 w-4 h-4 bg-press-muted rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-press-on-accent"
                ></div>
              </div>
            </label>

            {#if includeCoverImage}
              <div>
                <label for="cover-image" class="block text-press-eyebrow text-press-muted mb-1.5"
                  >Cover image</label
                >
                <div class="flex gap-2">
                  <input
                    id="cover-image"
                    type="text"
                    readonly
                    value={coverImagePath}
                    placeholder="Select an image..."
                    class="flex-1 bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg px-3 py-2.5 focus:outline-none focus:border-press-accent cursor-pointer truncate"
                    onclick={selectCoverImage}
                  />
                  <Tooltip text="Browse" position="top">
                    <button
                      type="button"
                      onclick={selectCoverImage}
                      class="px-3 py-2.5 bg-press-sunken text-press-muted rounded-lg hover:bg-press-sunken hover:text-press-text transition-colors border border-press-border"
                      aria-label="Select cover image"
                    >
                      <ImageIcon class="w-5 h-5" />
                    </button>
                  </Tooltip>
                </div>
              </div>
            {/if}
          </div>
        </fieldset>

        <!-- Save Location -->
        <div>
          <label
            for="epub-destination"
            class="block text-press-ui font-medium text-press-muted mb-2"
          >
            Save Location
          </label>
          <div class="flex gap-2">
            <input
              id="epub-destination"
              type="text"
              readonly
              value={epubFilePath}
              placeholder="Choose where to save..."
              class="flex-1 bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg px-3 py-2.5 focus:outline-none focus:border-press-accent cursor-pointer truncate"
              onclick={selectEpubFile}
            />
            <Tooltip text="Browse" position="top">
              <button
                type="button"
                onclick={selectEpubFile}
                class="px-3 py-2.5 bg-press-sunken text-press-muted rounded-lg hover:bg-press-sunken hover:text-press-text transition-colors border border-press-border"
                aria-label="Choose save location"
              >
                <Book class="w-5 h-5" />
              </button>
            </Tooltip>
          </div>
        </div>
      {/if}

      {#if exportFormat !== "custom"}
        <!-- Snapshot Option (shown for both formats) -->
        <div class="pt-2 border-t border-press-border/50">
          <label
            class="flex items-center justify-between p-3 bg-press-sunken rounded-lg cursor-pointer hover:bg-press-sunken transition-colors"
          >
            <div>
              <span class="text-press-ui text-press-text">Create snapshot before exporting</span>
              <p class="text-press-eyebrow text-press-muted mt-0.5">
                Save a backup of your current work
              </p>
            </div>
            <div class="relative">
              <input type="checkbox" bind:checked={createSnapshot} class="peer sr-only" />
              <div
                class="w-10 h-6 bg-press-sunken rounded-full peer-checked:bg-press-accent transition-colors"
              ></div>
              <div
                class="absolute left-1 top-1 w-4 h-4 bg-press-muted rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-press-on-accent"
              ></div>
            </div>
          </label>
        </div>
      {/if}
      <!-- Error Message -->
      {#if error}
        <div class="p-3 bg-press-error-wash border border-press-error rounded-lg">
          <p class="text-press-ui text-press-error">{error}</p>
        </div>
      {/if}
    </div>

    <!-- Footer -->
    <div
      class="flex items-center justify-end gap-3 px-5 py-4 border-t border-press-border flex-shrink-0 bg-press-surface"
    >
      <button
        type="button"
        onclick={onClose}
        class="px-4 py-2 text-press-ui text-press-muted hover:text-press-text transition-colors rounded-lg hover:bg-press-sunken"
        disabled={exporting}
      >
        Cancel
      </button>
      <button
        type="button"
        data-testid="export-confirm"
        onclick={handleExport}
        class="px-5 py-2 text-press-ui font-medium bg-press-accent text-press-on-accent rounded-lg hover:bg-press-accent-text transition-colors disabled:cursor-not-allowed flex items-center gap-2"
        disabled={!canExport || exporting}
      >
        {#if exporting}
          <Loader2 class="w-4 h-4 animate-spin" />
          Exporting...
        {:else if exportFormat === "custom"}
          <Settings2 class="w-4 h-4" /> Open workspace
        {:else}
          <FileDown class="w-4 h-4" />
          Export
        {/if}
      </button>
    </div>
  </div>
</div>

{#if showMatchPreview && currentProject.value}
  <ScrivenerMatchDialog
    projectId={currentProject.value.id}
    scrivPath={scrivenerPath}
    onConfirm={handleExport}
    onCancel={() => (showMatchPreview = false)}
  />
{/if}
