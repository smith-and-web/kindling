<!--
  ExportDialog.svelte - Export configuration dialog

  Allows users to configure and initiate project exports:
  - Format selection (Markdown, Longform, Word Document, or ePub)
  - Scope selection based on context (project/chapter/scene)
  - Options like beat markers, synopsis, page breaks
  - Destination folder/file picker
-->
<script lang="ts">
  import DialogHeader from "./DialogHeader.svelte";
  import { onMount } from "svelte";
  import {
    decodeProfiles,
    profileStorageKey,
    starterProfiles,
    formatLabels,
    type ExportProfile,
  } from "../utils/exportPrototype";
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "../utils/nativeDialog";
  import {
    ArrowRight,
    Download,
    TriangleAlert,
    Loader2,
    FileText,
    AlignLeft,
    BookOpen,
    Book,
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
  import { modalFocus, ownsEnterKey } from "../utils/modalFocus";

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
    { id: "custom", label: "Custom", detail: "Your saved export profiles", icon: Settings2 },
  ] as const;

  // Formats grouped by what the writer is doing with the export.
  const showNovelWriter = $derived(
    scope === "project" && currentProject.value?.project_type !== "screenplay"
  );
  const formatById = (id: (typeof exportFormats)[number]["id"]) =>
    exportFormats.find((format) => format.id === id)!;
  const formatGroups = $derived([
    {
      id: "share",
      label: "Share a document",
      formats: [formatById("docx"), formatById("epub"), formatById("treatment")],
    },
    {
      id: "apps",
      label: "Continue in another app",
      formats: [
        formatById("scrivener"),
        ...(showNovelWriter ? [formatById("novelwriter")] : []),
        formatById("longform"),
        formatById("markdown"),
      ],
    },
  ]);

  // The profile dropdown stands in for a "Custom" format: choosing a profile
  // switches to it, and "None" returns to the last standard format.
  let lastStandardFormat = $state<Exclude<typeof exportFormat, "custom">>("docx");
  $effect(() => {
    if (exportFormat !== "custom") lastStandardFormat = exportFormat;
  });

  function chooseProfile(id: string) {
    if (!id) {
      exportFormat = lastStandardFormat;
      return;
    }
    customProfileId = id;
    exportFormat = "custom";
    rememberCustomProfile();
  }

  const HEADING_LABELS: Record<ExportProfile["chapterHeading"], string> = {
    number_title: "Number and title",
    number: "Number only",
    title: "Title only",
    none: "No chapter headings",
  };

  /** What the profile will produce, in the terms the workspace uses. */
  function profileSummary(p: ExportProfile): [string, string][] {
    const rows: [string, string][] = [["Output", formatLabels[p.format]]];
    if (p.format === "docx" || p.format === "epub" || p.format === "html" || p.format === "txt") {
      const spacing =
        p.lineSpacing === 1 ? "single" : p.lineSpacing === 2 ? "double" : `${p.lineSpacing}×`;
      if (p.format !== "txt")
        rows.push(["Text & page", `${p.font} · ${p.fontSize} pt · ${spacing}`]);
      rows.push([
        "Headings & breaks",
        `${HEADING_LABELS[p.chapterHeading]}${p.chapterHeading.startsWith("number") && p.numberStyle === "roman" ? " (roman)" : ""} · break “${p.separator || "blank line"}”`,
      ]);
    }
    const extras = [
      p.titlePage && "title page",
      p.contents && "contents",
      p.synopses && "synopses",
      p.beatHeadings && "beat headings",
    ].filter(Boolean);
    rows.push([
      "Content",
      `${p.selection === "all" ? "Whole manuscript" : "Selected chapters"}${extras.length ? ` · ${extras.join(", ")}` : ""}`,
    ]);
    return rows;
  }

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
    { value: "number_only", label: "Number only", example: "CHAPTER ONE" },
    { value: "number_and_title", label: "Number and title", example: "CHAPTER ONE: THE BEGINNING" },
    { value: "title_only", label: "Title only", example: "THE BEGINNING" },
    { value: "number_arabic", label: "Arabic numeral", example: "CHAPTER 1" },
    {
      value: "number_arabic_and_title",
      label: "Arabic and title",
      example: "CHAPTER 1: THE BEGINNING",
    },
  ];

  // Scene break style options
  const sceneBreakStyles: { value: SceneBreakStyle; label: string; example: string }[] = [
    { value: "hash", label: "Hash mark", example: "#" },
    { value: "asterisks", label: "Three asterisks", example: "* * *" },
    { value: "asterism", label: "Asterism", example: "⁂" },
    { value: "blank_line", label: "Blank line", example: "(blank)" },
  ];

  // Font family options
  const fontFamilies: { value: FontFamily; label: string }[] = [
    { value: "courier_new", label: "Courier New" },
    { value: "times_new_roman", label: "Times New Roman" },
  ];

  // Line spacing options
  const lineSpacingOptions: { value: LineSpacingOption; label: string }[] = [
    { value: "single", label: "Single" },
    { value: "one_and_half", label: "1.5 lines" },
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

  /** Why Export is unavailable, said out loud rather than left to a greyed button. */
  const blockedReason = $derived.by(() => {
    if (canExport) return null;
    if (exportFormat === "custom") {
      return profileLoadFailed
        ? "Export profiles couldn’t be loaded."
        : "Choose an export profile.";
    }
    if (exportFormat === "novelwriter" && scope !== "project") {
      return "novelWriter exports the whole project.";
    }
    if (exportFormat === "epub" && epubFilePath.length > 0 && includeCoverImage) {
      return "Choose a cover image, or turn off the cover.";
    }
    return exportFormat === "markdown" ||
      exportFormat === "longform" ||
      exportFormat === "novelwriter"
      ? "Choose a destination folder before exporting."
      : "Choose where to save before exporting.";
  });

  const exportActionLabel = $derived(
    {
      docx: "Export .docx",
      epub: "Export .epub",
      markdown: "Export Markdown",
      longform: "Export Longform",
      novelwriter: "Export novelWriter",
      scrivener: "Export to Scrivener",
      treatment: treatmentFormat === "txt" ? "Export .txt" : "Export .docx",
      custom: "Open workspace",
    }[exportFormat] ?? "Export"
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
    // Escape and focus containment are handled by modalFocus. Enter is the default
    // action only where it does nothing else: buttons, links, selects and textareas
    // act on Enter themselves, so Enter on Cancel must not export.
    if (event.key !== "Enter" || event.isComposing || ownsEnterKey(event.target)) return;
    if (canExport && !exporting) {
      event.preventDefault();
      handleExport();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }
</script>

{#snippet formatOption(format: (typeof exportFormats)[number])}
  <label class="export-option" class:is-selected={exportFormat === format.id}>
    <input
      type="radio"
      name="format"
      data-testid={`export-format-${format.id}`}
      value={format.id}
      bind:group={exportFormat}
      onchange={() => {
        if (format.id === "custom") rememberCustomProfile();
      }}
    />
    <span class="export-option-name">{format.label}</span>
    <small>{format.detail}</small>
  </label>
{/snippet}

<!-- Backdrop. Escape is the keyboard equivalent of the backdrop click; modalFocus handles it. -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="dialog-scrim"
  use:modalFocus={{
    onEscape: onClose,
    onKeydown: handleKeydown,
    initialFocus: "input[name='format']:checked",
  }}
  onclick={handleBackdropClick}
  role="dialog"
  aria-modal="true"
  aria-labelledby="export-dialog-title"
  tabindex="-1"
>
  <!-- Dialog -->
  <div class="app-dialog-surface ka-dialog-default dialog-shell classic-export">
    <DialogHeader
      title={`Export ${scopeTitle}`}
      titleId="export-dialog-title"
      subtitle="Choose a format and configure its options"
      {onClose}
      closeLabel="Close"
      closeTestId="export-close"
    >
      {#if scope === "project" && (loadingWordCount || formattedWordCount)}
        <span class="ka-badge">{loadingWordCount ? "Counting words…" : formattedWordCount}</span>
      {/if}
    </DialogHeader>

    <!-- Content -->
    <div class="ka-dialog-body classic-export-body">
      <!-- Format Selection - Card Style -->
      <fieldset class="export-picker">
        <legend class="ka-group-title">Format</legend>
        <div class="export-picker-groups">
          {#each formatGroups as group (group.id)}
            <div
              class="export-picker-group"
              role="group"
              aria-labelledby={`format-group-${group.id}`}
            >
              <p id={`format-group-${group.id}`} class="export-picker-label">{group.label}</p>
              {#each group.formats as format (format.id)}
                {@render formatOption(format)}
              {/each}
            </div>
          {/each}
        </div>
        <div class="ka-field od-field export-profile">
          <label for="custom-export-profile">Export profile</label>
          <select
            id="custom-export-profile"
            value={exportFormat === "custom" ? customProfileId : ""}
            onchange={(event) => chooseProfile(event.currentTarget.value)}
            disabled={profileLoadFailed}
            aria-describedby="custom-export-profile-help"
          >
            <option value="">None</option>
            {#each customProfiles as profile (profile.id)}<option value={profile.id}
                >{profile.name}</option
              >{/each}
          </select>
          {#if profileLoadFailed}<p class="ka-error" role="alert">
              Saved profiles could not be loaded. Close and reopen Export to try again.
            </p>
          {:else if exportFormat === "custom" && selectedCustomProfile}
            {@const summary = profileSummary(selectedCustomProfile)}
            <div id="custom-export-profile-help" class="export-profile-summary">
              {#if selectedCustomProfile.description}<p class="ka-help">
                  {selectedCustomProfile.description}
                </p>{/if}
              <dl class="ka-facts">
                {#each summary as [term, value] (term)}
                  <div>
                    <dt>{term}</dt>
                    <dd>{value}</dd>
                  </div>
                {/each}
              </dl>
              <p class="ka-help">This project remembers your chosen profile.</p>
            </div>
          {:else}
            <p id="custom-export-profile-help" class="ka-help">
              Or use one of your saved export profiles, set up in the export workspace.
            </p>
          {/if}
        </div>
      </fieldset>

      {#if exportFormat === "docx"}
        <!-- DOCX Options Section -->
        <fieldset>
          <legend class="ka-group-title">Document structure </legend>

          <!-- Toggle Options -->
          <div class="export-choices">
            <label class="ka-check"
              ><input type="checkbox" bind:checked={includeTitlePage} /> Include title page</label
            >

            <label class="ka-check"
              ><input type="checkbox" bind:checked={pageBreaksBetweenChapters} /> Page breaks between
              chapters</label
            >

            <label class="ka-check"
              ><input type="checkbox" bind:checked={includeBeatMarkers} /> Include beat markers as headings</label
            >

            <label class="ka-check"
              ><input type="checkbox" bind:checked={includeSynopsis} /> Include scene synopses</label
            >
          </div>

          <!-- Dropdown Selects -->
          <div class="export-grid">
            <!-- Chapter Heading Style -->
            <div class="ka-field od-field">
              <label for="chapter-heading-style"> Chapter heading </label>
              <select id="chapter-heading-style" bind:value={chapterHeadingStyle}>
                {#each chapterHeadingStyles as style (style.value)}
                  <option value={style.value}>{style.label}</option>
                {/each}
              </select>
              <p class="ka-help">
                Appears as: {chapterHeadingStyles.find((s) => s.value === chapterHeadingStyle)
                  ?.example}
              </p>
            </div>

            <!-- Scene Break Style -->
            <div class="ka-field od-field">
              <label for="scene-break-style"> Scene break </label>
              <select id="scene-break-style" bind:value={sceneBreakStyle}>
                {#each sceneBreakStyles as style (style.value)}
                  <option value={style.value}>{style.label}</option>
                {/each}
              </select>
              <p class="ka-help">
                Appears as: {sceneBreakStyles.find((s) => s.value === sceneBreakStyle)?.example}
              </p>
            </div>
          </div>
        </fieldset>

        <!-- Typography Section -->
        <fieldset>
          <legend class="ka-group-title">Typography </legend>
          <div class="export-grid">
            <!-- Font Family -->
            <div class="ka-field od-field">
              <label for="font-family"> Font </label>
              <select id="font-family" bind:value={fontFamily}>
                {#each fontFamilies as font (font.value)}
                  <option value={font.value}>{font.label}</option>
                {/each}
              </select>
            </div>

            <!-- Line Spacing -->
            <div class="ka-field od-field">
              <label for="line-spacing"> Line spacing </label>
              <select id="line-spacing" bind:value={lineSpacing}>
                {#each lineSpacingOptions as spacing (spacing.value)}
                  <option value={spacing.value}>{spacing.label}</option>
                {/each}
              </select>
            </div>
          </div>
        </fieldset>

        <!-- Save Location -->
        <div class="ka-field od-field">
          <label for="docx-destination"> Save location </label>
          <div class="export-location">
            <input
              id="docx-destination"
              type="text"
              readonly
              value={docxFilePath}
              placeholder="No location chosen"
              onclick={selectDocxFile}
            />
            <button
              type="button"
              onclick={selectDocxFile}
              class="ka-button ka-button--secondary"
              aria-label="Choose save location"
            >
              Choose…
            </button>
          </div>
        </div>
      {:else if exportFormat === "markdown"}
        <!-- Markdown Options -->
        <fieldset>
          <legend class="ka-group-title">Options </legend>

          <div class="export-choices">
            <label class="ka-check"
              ><input type="checkbox" bind:checked={includeBeatMarkers} /> Include beat markers as headings</label
            >

            <label class="ka-check"
              ><input type="checkbox" bind:checked={deleteExisting} /> Delete existing export folder</label
            >
          </div>
        </fieldset>

        <!-- Export Name -->
        <div class="ka-field od-field">
          <label for="export-name"> Export name </label>
          <input
            id="export-name"
            type="text"
            bind:value={exportName}
            placeholder="Enter export folder name…"
          />
          <p class="ka-help">
            Folder: <span>{exportName.trim() || currentProject.value?.name || "Project"}</span>
          </p>
        </div>

        <!-- Destination Folder -->
        <div class="ka-field od-field">
          <label for="destination"> Destination folder </label>
          <div class="export-location">
            <input
              id="destination"
              type="text"
              readonly
              value={outputPath}
              placeholder="Select a folder…"
              onclick={selectDestination}
            />
            <button
              type="button"
              onclick={selectDestination}
              class="ka-button ka-button--secondary"
              aria-label="Browse for folder"
            >
              Choose…
            </button>
          </div>
        </div>
      {:else if exportFormat === "longform"}
        <!-- Longform Options -->
        <fieldset>
          <legend class="ka-group-title">Options </legend>

          <div class="export-choices">
            <label class="ka-check"
              ><input type="checkbox" bind:checked={deleteExisting} /> Delete existing export folder</label
            >
          </div>
        </fieldset>

        <!-- Export Name -->
        <div class="ka-field od-field">
          <label for="export-name-longform"> Export name </label>
          <input
            id="export-name-longform"
            type="text"
            bind:value={exportName}
            placeholder="Enter project name…"
          />
          <p class="ka-help">
            Folder: <span>{exportName.trim() || currentProject.value?.name || "Project"}</span>
            · Index:
            <span>{exportName.trim() || currentProject.value?.name || "Project"}.md</span>
          </p>
        </div>

        <!-- Destination Folder -->
        <div class="ka-field od-field">
          <label for="destination-longform"> Destination folder </label>
          <div class="export-location">
            <input
              id="destination-longform"
              type="text"
              readonly
              value={outputPath}
              placeholder="Select a folder…"
              onclick={selectDestination}
            />
            <button
              type="button"
              onclick={selectDestination}
              class="ka-button ka-button--secondary"
              aria-label="Browse for folder"
            >
              Choose…
            </button>
          </div>
        </div>
      {:else if exportFormat === "treatment"}
        <!-- Treatment Options -->
        <fieldset>
          <legend class="ka-group-title">Detail level </legend>
          <div class="export-choices">
            <label class="ka-check export-check" class:is-selected={treatmentLevel === "one_page"}>
              <input
                type="radio"
                name="treatment-level"
                value="one_page"
                bind:group={treatmentLevel}
              />
              <span class="export-check-text"
                ><span>One-Page</span><small>Title, logline, and a short synopsis per act</small
                ></span
              >
            </label>
            <label class="ka-check export-check" class:is-selected={treatmentLevel === "five_page"}>
              <input
                type="radio"
                name="treatment-level"
                value="five_page"
                bind:group={treatmentLevel}
              />
              <span class="export-check-text"
                ><span>Five-Page</span><small>Act summaries with key scene descriptions</small
                ></span
              >
            </label>
            <label class="ka-check export-check" class:is-selected={treatmentLevel === "full"}>
              <input type="radio" name="treatment-level" value="full" bind:group={treatmentLevel} />
              <span class="export-check-text"
                ><span>Full Treatment</span><small>Every scene synopsis and beat description</small
                ></span
              >
            </label>
          </div>
        </fieldset>

        <!-- Output Format -->
        <fieldset class="ka-segments">
          <legend class="ka-group-title">Output format</legend>
          <div class="ka-segment-track">
            {#each [{ id: "docx", label: ".docx" }, { id: "txt", label: ".txt" }] as option (option.id)}
              <label class="ka-segment" class:ka-selected={treatmentFormat === option.id}>
                <input
                  type="radio"
                  name="treatment-format"
                  value={option.id}
                  bind:group={treatmentFormat}
                />
                {option.label}
              </label>
            {/each}
          </div>
        </fieldset>

        <!-- Save Location -->
        <div class="ka-field od-field">
          <label for="treatment-destination"> Save location </label>
          <div class="export-location">
            <input
              id="treatment-destination"
              type="text"
              readonly
              value={treatmentFilePath}
              placeholder="No location chosen"
              onclick={selectTreatmentFile}
            />
            <button
              type="button"
              onclick={selectTreatmentFile}
              class="ka-button ka-button--secondary"
              aria-label="Choose save location"
            >
              Choose…
            </button>
          </div>
        </div>
      {:else if exportFormat === "novelwriter"}
        <div class="export-section">
          <p class="ka-help">
            Export a complete project for novelWriter 26.2 or newer. Choose an empty folder.
          </p>
          <label class="ka-check"
            ><input
              type="checkbox"
              data-testid="novelwriter-beat-comments"
              bind:checked={novelwriterBeatComments}
            /> Include beat comments</label
          >
          <p class="ka-help">
            Turning off beat comments disables beat-level sync. Page-mode prose syncs as a whole
            scene.
          </p>
          <label class="ka-check"
            ><input
              type="checkbox"
              data-testid="novelwriter-notes"
              bind:checked={novelwriterNotes}
            /> Include characters, locations and notes</label
          >
          <div class="ka-field od-field">
            <label for="novelwriter-destination">Destination folder</label>
            <div class="export-location">
              <input
                id="novelwriter-destination"
                readonly
                value={novelwriterPath}
                placeholder="Choose an empty folder"
              />
              <button
                type="button"
                onclick={selectNovelWriterPath}
                aria-label="Choose novelWriter destination folder"
                class="ka-button ka-button--secondary">Choose…</button
              >
            </div>
          </div>
          <details class="ka-disclosure export-details">
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
          <legend class="ka-group-title">Export mode </legend>
          <div class="export-choices">
            <label class="ka-check export-check" class:is-selected={scrivenerMode === "create_new"}>
              <input
                type="radio"
                name="scrivener-mode"
                value="create_new"
                bind:group={scrivenerMode}
              />
              <span class="export-check-text"
                ><span>Create New</span><small>Build a fresh .scriv project from your outline</small
                ></span
              >
            </label>
            <label class="ka-check export-check" class:is-selected={scrivenerMode === "update"}>
              <input type="radio" name="scrivener-mode" value="update" bind:group={scrivenerMode} />
              <span class="export-check-text"
                ><span>Update Existing</span><small
                  >Write prose back into an existing .scriv bundle</small
                ></span
              >
            </label>
          </div>
        </fieldset>

        {#if scrivenerMode === "update"}
          <!-- Update-mode-specific options -->
          <fieldset>
            <legend class="ka-group-title">Update options </legend>
            <div class="export-choices">
              <label class="ka-check export-check">
                <input type="checkbox" bind:checked={scrivenerBackup} />
                <span class="export-check-text"
                  ><span>Backup before updating</span><small
                    >Creates a timestamped copy of the .scriv bundle</small
                  ></span
                >
              </label>

              <label class="ka-check export-check">
                <input type="checkbox" bind:checked={scrivenerIncludeUnmatched} />
                <span class="export-check-text"
                  ><span>Include unmatched scenes</span><small
                    >Create new Scrivener documents for scenes without matches</small
                  ></span
                >
              </label>
            </div>
          </fieldset>
        {/if}

        <p class="ka-help">
          Characters, locations and scene references are not included in Scrivener exports.
        </p>

        <!-- Save/Select Location -->
        <div class="ka-field od-field">
          <label for="scrivener-destination">
            {scrivenerMode === "create_new" ? "Save Location" : "Select .scriv Bundle"}
          </label>
          <div class="export-location">
            <input
              id="scrivener-destination"
              type="text"
              readonly
              value={scrivenerPath}
              placeholder={scrivenerMode === "create_new"
                ? "Choose where to save…"
                : "Select existing .scriv folder…"}
              onclick={selectScrivenerPath}
            />
            <button
              type="button"
              onclick={selectScrivenerPath}
              class="ka-button ka-button--secondary"
              aria-label={scrivenerMode === "create_new"
                ? "Choose save location"
                : "Select .scriv bundle"}
            >
              Choose…
            </button>
          </div>
        </div>
      {:else if exportFormat === "epub"}
        <!-- EPUB Options -->
        <fieldset>
          <legend class="ka-group-title">Metadata </legend>
          <div class="export-section">
            <div class="ka-field od-field">
              <label for="epub-title">Title</label>
              <input id="epub-title" type="text" bind:value={epubTitle} placeholder="Book title" />
            </div>
            <div class="ka-field od-field">
              <label for="epub-author">Author</label>
              <input
                id="epub-author"
                type="text"
                bind:value={epubAuthor}
                placeholder="Author name"
              />
            </div>
            <div class="ka-field od-field">
              <label for="epub-description">Description</label>
              <textarea
                id="epub-description"
                rows="3"
                bind:value={epubDescription}
                placeholder="Short blurb or summary"
              ></textarea>
            </div>
            <div class="ka-field od-field">
              <label for="epub-language">Language</label>
              <input id="epub-language" type="text" bind:value={epubLanguage} placeholder="en" />
              <p class="ka-help">Use ISO 639-1 codes (e.g., en, es).</p>
            </div>
          </div>
        </fieldset>

        <fieldset>
          <legend class="ka-group-title">Content </legend>
          <div class="export-choices">
            <label class="ka-check"
              ><input type="checkbox" bind:checked={includeBeatMarkers} /> Include beat markers as headings</label
            >
            <label class="ka-check"
              ><input type="checkbox" bind:checked={includeSynopsis} /> Include scene synopses</label
            >
          </div>
        </fieldset>

        <fieldset>
          <legend class="ka-group-title">Styling </legend>
          <div class="ka-field od-field">
            <label for="epub-theme">Theme</label>
            <select id="epub-theme" bind:value={epubTheme}>
              {#each epubThemeOptions as theme (theme.value)}
                <option value={theme.value}>{theme.label}</option>
              {/each}
            </select>
          </div>
        </fieldset>

        <fieldset>
          <legend class="ka-group-title">Cover </legend>
          <div class="export-section">
            <label class="ka-check"
              ><input type="checkbox" bind:checked={includeCoverImage} /> Include cover image</label
            >

            {#if includeCoverImage}
              <div class="ka-field od-field">
                <label for="cover-image">Cover image</label>
                <div class="export-location">
                  <input
                    id="cover-image"
                    type="text"
                    readonly
                    value={coverImagePath}
                    placeholder="Select an image…"
                    onclick={selectCoverImage}
                  />
                  <button
                    type="button"
                    onclick={selectCoverImage}
                    class="ka-button ka-button--secondary"
                    aria-label="Select cover image"
                  >
                    Choose…
                  </button>
                </div>
              </div>
            {/if}
          </div>
        </fieldset>

        <!-- Save Location -->
        <div class="ka-field od-field">
          <label for="epub-destination"> Save location </label>
          <div class="export-location">
            <input
              id="epub-destination"
              type="text"
              readonly
              value={epubFilePath}
              placeholder="No location chosen"
              onclick={selectEpubFile}
            />
            <button
              type="button"
              onclick={selectEpubFile}
              class="ka-button ka-button--secondary"
              aria-label="Choose save location"
            >
              Choose…
            </button>
          </div>
        </div>
      {/if}

      {#if exportFormat !== "custom"}
        <!-- Snapshot Option (shown for both formats) -->
        <div class="export-snapshot">
          <label class="ka-check export-check">
            <input type="checkbox" bind:checked={createSnapshot} />
            <span class="export-check-text"
              ><span>Create snapshot before exporting</span><small
                >Save a backup of your current work</small
              ></span
            >
          </label>
        </div>
      {/if}
      <!-- Error Message -->
      {#if error}
        <div class="ka-notice ka-notice--error od-row-top" role="alert">
          <TriangleAlert class="w-5 h-5" aria-hidden="true" />
          <div class="od-field od-fill">
            <strong>Couldn’t export</strong>
            <p>{error}</p>
          </div>
        </div>
      {/if}
    </div>

    <footer class="ka-dialog-footer">
      {#if blockedReason && !exporting}
        <p class="ka-help ka-dialog-footer-start" id="export-blocked">{blockedReason}</p>
      {:else if exportFormat === "custom"}
        <p class="ka-help ka-dialog-footer-start">
          Opens the export workspace with your saved settings.
        </p>
      {/if}
      <button
        type="button"
        onclick={onClose}
        class="ka-button ka-button--secondary"
        disabled={exporting}
      >
        Cancel
      </button>
      <button
        type="button"
        data-testid="export-confirm"
        onclick={handleExport}
        class="ka-button"
        disabled={!canExport || exporting}
        aria-busy={exporting || undefined}
        aria-describedby={blockedReason && !exporting ? "export-blocked" : undefined}
      >
        {#if exporting}
          <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
          Exporting…
        {:else if exportFormat === "custom"}
          Open workspace <ArrowRight class="w-5 h-5" aria-hidden="true" />
        {:else}
          <Download class="w-5 h-5" aria-hidden="true" />
          {exportActionLabel}
        {/if}
      </button>
    </footer>
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

<style>
  .classic-export-body {
    display: grid;
    gap: var(--space-m);
  }
  .classic-export-body > :global(fieldset) {
    display: grid;
    gap: var(--space-s);
    min-width: 0;
    margin: 0;
    padding: 0;
    border: 0;
  }
  /* A legend interrupts its fieldset's top border (the rule ran beside the
     group title), so the hairline closes the group *before* each fieldset. */
  .classic-export-body > :global(*:has(+ fieldset:not(.ka-segments))) {
    padding-bottom: var(--space-m);
    border-bottom: var(--border-hair);
  }
  .classic-export-body > :global(fieldset > legend.ka-group-title) {
    margin-bottom: var(--space-xs);
  }
  .classic-export-body > :global(fieldset.ka-segments) {
    padding-top: 0;
    border-top: 0;
  }
  /* Format picker: grouped radio rows (label, then its file type), the same
     hairline-row pattern as the start screen's import list. */
  .export-picker {
    display: grid;
    gap: var(--space-2xs);
  }
  .export-picker-groups {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    column-gap: var(--space-m);
    align-items: start;
  }
  .export-picker-group {
    display: grid;
    align-content: start;
  }
  .export-picker-label {
    margin: 0;
    padding-block: var(--space-2xs);
    border-bottom: var(--border-hair);
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .export-option {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    min-height: var(--control-target);
    padding-inline: var(--space-2xs);
    border-bottom: var(--border-hair);
    cursor: pointer;
    font: var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .export-option small {
    margin-left: auto;
    font: var(--text-small) / 1.4 var(--font-ui);
    color: var(--color-text-muted);
    text-align: right;
  }
  .export-option.is-selected .export-option-name {
    font-weight: 600;
  }
  .export-option:focus-within {
    outline: 2px solid var(--color-accent-text);
    outline-offset: -2px;
    border-radius: var(--radius-xs);
  }
  .export-option:has(input:focus:not(:focus-visible)) {
    outline: none;
  }
  @media (hover: hover) {
    .export-option:hover {
      background: var(--color-surface-sunken);
    }
  }
  .export-profile {
    margin-top: var(--space-xs);
  }
  .export-profile-summary {
    display: grid;
    gap: var(--space-2xs);
  }
  .export-profile-summary .ka-facts {
    margin: 0;
  }
  .export-choices {
    display: grid;
    gap: 0 var(--space-m);
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  }
  .export-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    align-items: start;
    gap: var(--space-s);
  }
  .export-section {
    display: grid;
    gap: var(--space-s);
  }
  .export-location {
    display: flex;
    gap: var(--space-2xs);
  }
  /* Filled by the native chooser: read-only, and it looks it. */
  .export-location :global(input) {
    flex: 1;
    min-width: 0;
    cursor: pointer;
    background-color: var(--color-surface);
    border-color: var(--color-border);
  }
  .export-check {
    align-items: flex-start;
    padding-block: var(--space-2xs);
  }
  .export-check :global(input) {
    margin-top: 2px;
  }
  .export-check-text {
    display: grid;
    gap: 2px;
  }
  .export-check-text small {
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .export-snapshot {
    padding-top: var(--space-s);
    border-top: var(--border-hair);
  }
  .export-details {
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
</style>
