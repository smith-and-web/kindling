<script lang="ts">
  import { countLabel } from "../utils/plural";
  import { onMount, tick, untrack } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "../utils/nativeDialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import {
    ArrowLeft,
    ArrowRight,
    BookOpen,
    Check,
    ChevronRight,
    Download,
    FileDown,
    FileText,
    Info,
    Loader2,
    RefreshCw,
    Search,
    TriangleAlert,
    X,
  } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { saveProseBefore } from "../utils/proseFlush";
  import { t } from "../i18n";
  import {
    compileWorkspaceDocument,
    exportFilename,
    outputExtension,
    isExchange,
    isPlain,
    decodeProfiles,
    formatLabels,
    manuscriptWords,
    profileStorageKey,
    profileForStorage,
    profileValidation,
    renderPreview,
    selectedChapters,
    starterProfiles,
    type ExportProfile,
    type PreviewChapter,
  } from "../utils/exportPrototype";

  let {
    scope,
    scopeId,
    onClose,
    onClassic,
  }: {
    scope: "project" | "chapter" | "scene";
    scopeId: string | null;
    onClose: () => void;
    onClassic: () => void;
  } = $props();
  const project = untrack(() => currentProject.value!);
  const starters = starterProfiles(project.name, project.author_pen_name ?? "");
  let profiles = $state<ExportProfile[]>(window.structuredClone(starters));
  let draft = $state<ExportProfile>(window.structuredClone(starters[0]));
  let ready = $state(false);
  let storageBlocked = $state(false);
  let storageError = $state("");
  let chapters = $state<PreviewChapter[]>([]);
  let loading = $state(true);
  let documentLoaded = $state(false);
  let error = $state("");
  let section = $state("overview");
  let query = $state("");
  let previewMode = $state<"rendered" | "source">("rendered");
  let previewChapter = $state("");
  let narrowPane = $state<"settings" | "preview">("settings");
  let saving = $state(false);
  let savedPath = $state("");
  let message = $state("");
  type Selection = Pick<ExportProfile, "selection" | "chapterIds" | "sceneId">;
  let contextSelection = $state<Selection | null>(null);
  const contextOverride = $derived(contextSelection !== null);
  let dialog: HTMLDialogElement;

  const allCategories = [
    {
      id: "overview",
      label: t("export.exportWorkspace.categoryOverview"),
      description: t("export.exportWorkspace.categoryOverviewDesc"),
    },
    {
      id: "content",
      label: t("export.exportWorkspace.categoryContent"),
      description: t("export.exportWorkspace.categoryContentDesc"),
    },
    {
      id: "headings",
      label: t("export.exportWorkspace.categoryHeadings"),
      description: t("export.exportWorkspace.categoryHeadingsDesc"),
    },
    {
      id: "text",
      label: t("export.exportWorkspace.categoryText"),
      description: t("export.exportWorkspace.categoryTextDesc"),
    },
    {
      id: "details",
      label: t("export.exportWorkspace.categoryDetails"),
      description: t("export.exportWorkspace.categoryDetailsDesc"),
    },
    {
      id: "files",
      label: t("export.exportWorkspace.categoryFiles"),
      description: t("export.exportWorkspace.categoryFilesDesc"),
    },
  ];
  const presetDescriptions: Record<string, string> = {
    docx: t("export.exportWorkspace.presetDocx"),
    epub: t("export.exportWorkspace.presetEpub"),
  };
  const settingsIndex = [
    {
      label: t("export.exportWorkspace.settingIncludedChapters"),
      terms: "selection scene manuscript scope",
      section: "content",
      control: "scope",
    },
    {
      label: t("export.exportWorkspace.settingSceneTitles"),
      terms: "notes summary outline",
      section: "content",
      control: "scene-titles",
    },
    {
      label: t("export.exportWorkspace.settingBeatHeadings"),
      terms: "outline prompts",
      section: "content",
      control: "beats",
    },
    {
      label: t("export.exportWorkspace.settingChapterHeadings"),
      terms: "title number roman numbering",
      section: "headings",
      control: "heading",
    },
    {
      label: t("export.exportWorkspace.settingSceneSeparator"),
      terms: "break marker hash asterisks",
      section: "headings",
      control: "separator",
    },
    {
      label: t("export.exportWorkspace.settingChapterBreaks"),
      terms: "new page",
      section: "headings",
      control: "chapter-breaks",
    },
    {
      label: t("export.exportWorkspace.settingFont"),
      terms: "typeface times courier georgia",
      section: "text",
      control: "font",
    },
    {
      label: t("export.exportWorkspace.settingSpacing"),
      terms: "double spaced single leading",
      section: "text",
      control: "spacing",
    },
    {
      label: t("export.exportWorkspace.settingIndent"),
      terms: "first line flush",
      section: "text",
      control: "indent",
    },
    {
      label: t("export.exportWorkspace.settingMargin"),
      terms: "a4 letter gutter page size",
      section: "text",
      control: "margin",
    },
    {
      label: t("export.exportWorkspace.settingTitle"),
      terms: "author identity subtitle",
      section: "details",
      control: "title",
    },
    {
      label: t("export.exportWorkspace.settingHeader"),
      terms: "running head author title",
      section: "details",
      control: "header",
    },
    {
      label: t("export.exportWorkspace.settingContents"),
      terms: "toc navigation",
      section: "details",
      control: "contents",
    },
    {
      label: t("export.exportWorkspace.settingHtml"),
      terms: "fragment full document css web",
      section: "files",
      control: "html-mode",
    },
    {
      label: t("export.exportWorkspace.settingFilename"),
      terms: "date destination name tokens",
      section: "files",
      control: "filename",
    },
  ];
  const exchange = $derived(isExchange(draft.format));
  const plain = $derived(isPlain(draft.format));
  const categories = $derived(
    allCategories.filter((c) =>
      exchange ? ["overview", "files"].includes(c.id) : plain ? c.id !== "text" : true
    )
  );
  $effect(() => {
    if (!categories.some((c) => c.id === section)) section = "overview";
  });
  const unsupported = $derived(
    draft.format === "novelwriter" && project.project_type === "screenplay"
      ? t("export.exportWorkspace.unsupportedNovelwriter")
      : ""
  );
  const scopeLabel = $derived(
    scope === "project"
      ? t("export.exportWorkspace.scopeProject")
      : scope === "chapter"
        ? t("export.exportWorkspace.scopeChapter")
        : t("export.exportWorkspace.scopeScene")
  );
  const searchResults = $derived(
    settingsIndex.filter(
      (item) =>
        categories.some((c) => c.id === item.section) &&
        `${item.label} ${item.terms}`.toLowerCase().includes(query.toLowerCase().trim())
    )
  );
  const selected = $derived(selectedChapters(chapters, draft));
  const sceneCount = $derived(selected.reduce((n, c) => n + c.scenes.length, 0));
  const wordCount = $derived(manuscriptWords(selected));
  const savedProfile = $derived(profiles.find((p) => p.id === draft.id));
  const dirty = $derived(JSON.stringify(draft) !== JSON.stringify(savedProfile));
  const visibleCategory = $derived(categories.find((c) => c.id === section) ?? categories[0]);
  const actualPreviewChapter = $derived(
    selected.some((c) => c.id === previewChapter) ? previewChapter : ""
  );
  const preview = $derived(
    renderPreview(chapters, draft, { chapterId: actualPreviewChapter || undefined })
  );
  const previewDocument = $derived(deskDocument(preview.document));
  // Set when the frame has rendered the current document (it loads async).
  let previewLoaded = $state("");
  const sourceText = $derived(
    plain ? compileWorkspaceDocument(chapters, draft).text : preview.output
  );
  const fileName = $derived(exportFilename(draft));
  const validationError = $derived(profileValidation(draft));
  const missingSelection = $derived(
    !exchange &&
      draft.selection === "selected" &&
      (draft.chapterIds.some((id) => !chapters.some((c) => c.id === id)) ||
        (draft.sceneId !== null &&
          !chapters.some((c) => c.scenes.some((s) => s.id === draft.sceneId))))
  );

  /** Keep the selected section in view when the nav scrolls (compact windows). */
  function keepVisible(node: HTMLElement, selected: boolean) {
    const reveal = (on: boolean) => {
      if (on) node.scrollIntoView?.({ block: "nearest" });
    };
    reveal(selected);
    return { update: reveal };
  }

  /** The preview sits on the app's desk: drop the export's own page backdrop
      (the exported file keeps it) so the sheet reads as paper in both themes.
      An unstyled HTML export has no sheet colours of its own, so it falls back
      to manuscript paper and ink; `:where()` keeps that fallback below any
      styling the export does carry. */
  function deskDocument(html: string): string {
    // The sandboxed frame can't read app tokens, so resolve them here.
    const probe = document.createElement("span");
    probe.style.boxShadow = "var(--shadow-prose)";
    probe.style.backgroundColor = "var(--color-prose-bg)";
    probe.style.color = "var(--color-prose-text)";
    document.body.appendChild(probe);
    const computed = getComputedStyle(probe);
    const resolved = (value: string, fallback: string) =>
      value && !value.includes("var(") ? value : fallback;
    const shadow = resolved(computed.boxShadow, "none");
    const paper = resolved(computed.backgroundColor, "Canvas");
    const ink = resolved(computed.color, "CanvasText");
    probe.remove();
    const mount = `body{background:transparent;padding:16px 24px 32px}:where(.manuscript){background:${paper};color:${ink};padding:32px}.manuscript{margin:0 auto;box-shadow:${shadow}}`;
    return html.replace("</head>", `<style>${mount}</style></head>`);
  }

  function mountDialog(node: HTMLDialogElement) {
    dialog = node;
    const previous = document.activeElement as HTMLElement | null;
    node.showModal();
    return {
      destroy() {
        node.close();
        previous?.focus();
      },
    };
  }

  onMount(() => {
    try {
      const raw = localStorage.getItem(profileStorageKey(project.id));
      if (raw) {
        const parsed = decodeProfiles(raw);
        profiles = parsed.profiles;
        draft = window.structuredClone(
          parsed.profiles.find((p) => p.id === parsed.activeId) ?? parsed.profiles[0]
        );
        const journal = JSON.parse(raw).draft;
        if (journal) {
          try {
            const checked = decodeProfiles(
              JSON.stringify({
                version: JSON.parse(raw).version,
                profiles: [journal],
                activeId: journal.id,
              })
            );
            if (journal.id === parsed.activeId && profiles.some((p) => p.id === journal.id))
              draft = checked.profiles[0];
          } catch {
            message = t("export.exportWorkspace.draftNotRestored");
          }
        }
      }
    } catch (e) {
      storageBlocked = true;
      storageError = t("export.exportWorkspace.profilesLoadFailed", { error: String(e) });
    }
    ready = true;
    // Preview the latest prose, not only what had reached the database. A draft
    // that cannot be saved is reported when exporting.
    void saveProseBefore(project.id, "exporting")
      .catch(() => {})
      .then(() => loadManuscript(true));
  });

  $effect(() => {
    if (!ready || storageBlocked) return;
    // Context-menu scope is temporary until explicitly saved or duplicated.
    const journal = contextSelection ? { ...draft, ...contextSelection } : draft;
    const data = JSON.stringify({ version: 2, profiles, activeId: draft.id, draft: journal });
    try {
      localStorage.setItem(profileStorageKey(project.id), data);
    } catch (e) {
      storageError = t("export.exportWorkspace.draftRetainFailed", { error: String(e) });
    }
  });

  async function loadManuscript(initial = false) {
    loading = true;
    documentLoaded = false;
    error = "";
    try {
      chapters = await invoke<PreviewChapter[]>("get_export_prototype_document", {
        projectId: project.id,
      });
      documentLoaded = true;
      if (initial && scope !== "project" && scopeId) {
        const chapter =
          scope === "chapter"
            ? chapters.find((c) => c.id === scopeId)
            : chapters.find((c) => c.scenes.some((s) => s.id === scopeId));
        contextSelection = {
          selection: draft.selection,
          chapterIds: [...draft.chapterIds],
          sceneId: draft.sceneId,
        };
        draft.selection = "selected";
        draft.chapterIds = [chapter?.id ?? scopeId];
        draft.sceneId = scope === "scene" ? scopeId : null;
      }
      if (initial) previewChapter = selectedChapters(chapters, draft)[0]?.id ?? "";
    } catch (e) {
      error = t("export.exportWorkspace.loadFailedWithError", { error: String(e) });
    } finally {
      loading = false;
    }
  }

  function saveProfile(rememberContext = true) {
    if (validationError) {
      message = validationError;
      return;
    }
    if (!draft.name.trim()) {
      message = t("export.exportWorkspace.nameRequired");
      section = "overview";
      return;
    }
    if (storageBlocked) {
      message = t("export.exportWorkspace.storageBlockedSave");
      return;
    }
    try {
      const candidate =
        !rememberContext && contextSelection
          ? { ...$state.snapshot(draft), ...$state.snapshot(contextSelection) }
          : $state.snapshot(draft);
      const saved = profileForStorage(candidate);
      const updated = profiles.map((p) => (p.id === draft.id ? saved : p));
      localStorage.setItem(
        profileStorageKey(project.id),
        JSON.stringify({ version: 2, profiles: updated, activeId: saved.id, draft: saved })
      );
      profiles = updated;
      draft = window.structuredClone(saved);
      contextSelection = null;
      storageError = "";
      message = t("export.exportWorkspace.profileSavedMessage", { name: draft.name });
    } catch (e) {
      storageError = t("export.exportWorkspace.saveFailed", { error: String(e) });
    }
  }

  function chooseProfile(id: string) {
    // Keep edits as a profile before switching, rather than losing an exploratory configuration.
    if (dirty) saveProfile(false);
    if (dirty) return;
    const next = profiles.find((p) => p.id === id);
    if (next) draft = window.structuredClone($state.snapshot(next));
    previewChapter = "";
    contextSelection = null;
    message = "";
  }

  // A switch can be refused (unsaved edits that can't be kept), so re-sync the
  // native control to the profile actually in use instead of the one clicked.
  function switchProfileFromSelect(select: HTMLSelectElement) {
    chooseProfile(select.value);
    select.value = draft.id;
  }

  function choosePreset(id: string, input: HTMLInputElement) {
    chooseProfile(id);
    input
      .closest("fieldset")
      ?.querySelectorAll<HTMLInputElement>('input[type="radio"]')
      .forEach((radio) => (radio.checked = radio.value === draft.id));
  }

  function duplicateProfile() {
    if (storageBlocked) return;
    try {
      const copy = {
        ...profileForStorage($state.snapshot(draft)),
        id: window.crypto.randomUUID(),
        name: `${draft.name} copy`,
      };
      const updated = [...profiles, copy];
      localStorage.setItem(
        profileStorageKey(project.id),
        JSON.stringify({
          version: 2,
          profiles: updated,
          activeId: copy.id,
          draft: copy,
        })
      );
      profiles = updated;
      draft = window.structuredClone(copy);
      contextSelection = null;
      storageError = "";
      section = "overview";
      message = t("export.exportWorkspace.copyCreated");
    } catch (e) {
      message = t("export.exportWorkspace.duplicateFailed", { error: String(e) });
    }
  }

  function selectChapter(id: string, checked: boolean) {
    draft.sceneId = null;
    draft.chapterIds = checked
      ? [...draft.chapterIds, id]
      : draft.chapterIds.filter((x) => x !== id);
  }

  async function findSetting(item: (typeof settingsIndex)[number]) {
    section = item.section;
    query = "";
    narrowPane = "settings";
    await tick();
    const target = dialog.querySelector<HTMLElement>(`#export-${item.control}`);
    if (target) {
      const details = target.closest("details");
      if (details) details.open = true;
      target.focus();
      target.scrollIntoView({ block: "center", behavior: "smooth" });
    } else message = t("export.exportWorkspace.settingUnavailable", { label: item.label });
  }

  async function saveExport() {
    if (validationError || unsupported) return;
    saving = true;
    error = "";
    savedPath = "";
    const profile = $state.snapshot(draft);
    const filename = exportFilename(profile);
    try {
      // The manuscript was read when the workspace opened; save and re-read it
      // so the file never carries stale prose.
      await saveProseBefore(project.id, "exporting");
      if (!isExchange(profile.format)) {
        // The loader reports its own failure; never export the old copy.
        await loadManuscript();
        if (!documentLoaded) return;
      }
      let path: string | null;
      if (["longform", "novelwriter"].includes(profile.format)) {
        const parent = await open({
          title: t("export.exportWorkspace.parentFolderTitle"),
          directory: true,
        });
        path = parent ? `${parent.replace(/[\\/]$/, "")}/${filename}` : null;
      } else {
        path = await save({
          title: t("export.exportWorkspace.saveDialogTitle", {
            format: formatLabels[profile.format],
          }),
          defaultPath: filename,
          filters: [{ name: formatLabels[profile.format], extensions: [outputExtension(profile)] }],
        });
      }
      if (!path) return;
      if (isExchange(profile.format)) {
        await invoke("export_workspace_exchange", {
          projectId: project.id,
          path,
          format: profile.format,
          options: {
            includeNotes: profile.includeNotes,
            includeBeatComments: profile.includeBeatComments,
            treatmentLevel: profile.treatmentLevel,
            treatmentFormat: profile.treatmentFormat,
          },
        });
      } else if (profile.format === "html") {
        await invoke("save_export_prototype_html", {
          path,
          html: renderPreview(chapters, profile).output,
        });
      } else {
        await invoke("export_workspace_document", {
          path,
          format: profile.format,
          document: compileWorkspaceDocument(chapters, profile),
        });
      }
      savedPath = path;
      message = t(
        isExchange(profile.format)
          ? "export.exportWorkspace.exportedExchange"
          : "export.exportWorkspace.exportedSelection",
        { format: formatLabels[profile.format] }
      );
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }
  async function chooseCover() {
    try {
      const path = await open({
        title: t("export.exportWorkspace.coverDialogTitle"),
        filters: [
          { name: t("export.exportWorkspace.coverFilterName"), extensions: ["png", "jpg", "jpeg"] },
        ],
      });
      if (path) draft.coverPath = path;
    } catch (e) {
      error = t("export.exportWorkspace.coverChooseFailed", { error: String(e) });
    }
  }

  // Reveal rather than open: the opener capability grants reveal-in-folder
  // (the classic export's success dialog uses it too) but not open-path, so
  // openPath was always refused at runtime.
  const revealFailed = t("export.exportWorkspace.revealFailed");
  async function openSaved() {
    try {
      await revealItemInDir(savedPath);
      if (error.startsWith(revealFailed)) error = "";
    } catch (e) {
      console.error("Failed to show export in folder:", e);
      error = t("export.exportWorkspace.revealFailedWithError", {
        error: e instanceof Error ? e.message : String(e),
      });
    }
  }
</script>

<dialog
  use:mountDialog
  aria-labelledby="export-workspace-title"
  class="export-workspace app-dialog-surface"
  data-testid="export-workspace"
  oncancel={(event) => {
    event.preventDefault();
    if (!saving) onClose();
  }}
>
  <header class="xw-appbar">
    <h3 id="export-workspace-title">{t("export.exportWorkspace.title")}</h3>
    <div class="xw-appbar-actions">
      <button type="button" class="ka-button ka-button--ghost" onclick={onClassic} disabled={saving}
        ><ArrowLeft class="w-5 h-5" aria-hidden="true" />{t(
          "export.exportWorkspace.backToExport"
        )}</button
      ><button
        type="button"
        class="ka-button ka-button--ghost ka-icon-button"
        aria-label={t("export.exportWorkspace.closeWorkspace")}
        title={t("export.exportWorkspace.closeWorkspace")}
        onclick={onClose}
        disabled={saving}><X class="w-5 h-5" aria-hidden="true" /></button
      >
    </div>
  </header>

  <div class="xw-tools">
    <div class="ka-field od-field xw-picker">
      <label for="export-profile">{t("export.exportWorkspace.profileLabel")}</label>
      <select
        id="export-profile"
        value={draft.id}
        onchange={(event) => switchProfileFromSelect(event.currentTarget)}
        >{#each profiles as profile (profile.id)}<option value={profile.id}>{profile.name}</option
          >{/each}</select
      >
    </div>
    <button
      type="button"
      class="ka-button ka-button--secondary"
      aria-label={t("export.exportWorkspace.duplicateProfile")}
      onclick={duplicateProfile}
      disabled={storageBlocked}>{t("export.exportWorkspace.duplicateProfile")}</button
    >
    <div class="ka-field od-field xw-picker">
      <label for="export-format">{t("export.exportWorkspace.outputFormat")}</label><select
        id="export-format"
        bind:value={draft.format}
        ><option value="docx">{t("export.exportWorkspace.formatDocx")}</option><option value="epub"
          >{t("export.exportWorkspace.formatEpub")}</option
        ><option value="html">{t("export.exportWorkspace.formatHtml")}</option><option
          value="markdown">{t("export.exportWorkspace.formatMarkdown")}</option
        ><option value="txt">{t("export.exportWorkspace.formatTxt")}</option><option
          value="longform">{t("export.exportWorkspace.formatLongform")}</option
        ><option value="scrivener">{t("export.exportWorkspace.formatScrivener")}</option><option
          value="novelwriter">{t("export.exportWorkspace.formatNovelwriter")}</option
        ><option value="treatment">{t("export.exportWorkspace.formatTreatment")}</option></select
      >
    </div>
    <div class="xw-status" role="status">
      {#if dirty}<span class="ka-badge ka-badge--warning"
          >{t("export.exportWorkspace.unsavedChanges")}</span
        >{:else}<span class="xw-ok"
          ><Check class="w-5 h-5" aria-hidden="true" />{t(
            "export.exportWorkspace.profileSaved"
          )}</span
        >{/if}
      <span class="xw-meta">{t("export.exportWorkspace.profilesLocal")}</span>
    </div>
  </div>

  <div
    class="ka-segment-track xw-switch"
    role="group"
    aria-label={t("export.exportWorkspace.workspaceView")}
  >
    <button
      type="button"
      class="ka-segment"
      class:ka-selected={narrowPane === "settings"}
      aria-pressed={narrowPane === "settings"}
      onclick={() => (narrowPane = "settings")}>{t("export.exportWorkspace.settings")}</button
    ><button
      type="button"
      class="ka-segment"
      class:ka-selected={narrowPane === "preview"}
      aria-pressed={narrowPane === "preview"}
      onclick={() => (narrowPane = "preview")}>{t("export.exportWorkspace.preview")}</button
    >
  </div>

  <div class="xw-body">
    <aside class="xw-nav" class:mobile-hidden={narrowPane === "preview"}>
      <div class="xw-search">
        <div class="ka-field od-field od-fill">
          <input
            type="search"
            aria-label={t("export.exportWorkspace.findSetting")}
            placeholder={t("export.exportWorkspace.searchPlaceholder")}
            bind:value={query}
          />
        </div>
        {#if query}<button
            type="button"
            class="ka-button ka-button--ghost ka-icon-button"
            aria-label={t("export.exportWorkspace.clearSearch")}
            title={t("export.exportWorkspace.clearSearch")}
            onclick={() => (query = "")}><X class="w-5 h-5" aria-hidden="true" /></button
          >{/if}
      </div>
      {#if query.trim()}
        {#if searchResults.length}
          <p class="xw-meta" role="status">
            {t("export.exportWorkspace.settingsFound", { count: searchResults.length })}
          </p>
          <ul class="xw-results">
            {#each searchResults as item (item.control)}<li>
                <button type="button" class="xw-row-button" onclick={() => findSetting(item)}
                  ><span class="od-fill">{item.label}</span><ChevronRight
                    class="w-5 h-5"
                    aria-hidden="true"
                  /></button
                >
              </li>{/each}
          </ul>
        {:else}
          <div class="ka-empty od-stack xw-empty" role="status">
            <Search class="w-7 h-7" aria-hidden="true" />
            <h4>{t("export.exportWorkspace.noSettingsMatch", { query: query.trim() })}</h4>
            <p>{t("export.exportWorkspace.searchHint")}</p>
          </div>
        {/if}
      {:else}
        <nav class="ka-tree" aria-label={t("export.exportWorkspace.exportSettings")}>
          <ul class="ka-tree-list">
            {#each categories as category (category.id)}<li>
                <button
                  type="button"
                  class:ka-tree-selected={section === category.id}
                  aria-current={section === category.id ? "page" : undefined}
                  use:keepVisible={section === category.id}
                  onclick={() => (section = category.id)}
                  ><span class="od-field od-fill"
                    ><span>{category.label}</span><small>{category.description}</small></span
                  ></button
                >
              </li>{/each}
          </ul>
        </nav>
        <p class="ka-help xw-nav-note">
          {t("export.exportWorkspace.navNote")}
        </p>
      {/if}
    </aside>

    <section
      class="settings-panel xw-pane"
      class:mobile-hidden={narrowPane === "preview"}
      aria-label={t("export.exportWorkspace.exportSettings")}
    >
      <div class="xw-pane-head">
        <p class="xw-meta">{formatLabels[draft.format]}</p>
        <h3>{visibleCategory.label}</h3>
        <p class="ka-help">{visibleCategory.description}.</p>
      </div>

      {#if section === "overview"}
        <div class="xw-group">
          <div class="ka-field od-field">
            <label for="export-name">{t("export.exportWorkspace.profileName")}</label><input
              id="export-name"
              bind:value={draft.name}
              maxlength="100"
              aria-invalid={!draft.name.trim() || undefined}
              aria-describedby="export-name-help"
            />
            <p class="ka-help" id="export-name-help">
              {t("export.exportWorkspace.profileNameHelp")}
            </p>
          </div>
        </div>
        <div class="xw-group">
          <fieldset class="xw-choice-list">
            <legend class="xw-label">{t("export.exportWorkspace.startingPoint")}</legend>
            {#each profiles.slice(0, 3) as preset (preset.id)}<div class="ka-check xw-choice">
                <input
                  id={`export-preset-${preset.id}`}
                  type="radio"
                  name="export-starting-point"
                  value={preset.id}
                  checked={draft.id === preset.id}
                  aria-describedby={`export-preset-${preset.id}-help`}
                  onchange={(event) => choosePreset(preset.id, event.currentTarget)}
                />
                <div class="od-field od-fill">
                  <label for={`export-preset-${preset.id}`}>{preset.name}</label>
                  <p class="ka-help" id={`export-preset-${preset.id}-help`}>
                    {presetDescriptions[preset.format] ??
                      t("export.exportWorkspace.presetFallback")}
                  </p>
                </div>
              </div>{/each}
          </fieldset>
        </div>
        <div class="xw-group">
          <h4 class="xw-label">{t("export.exportWorkspace.atAGlance")}</h4>
          <dl class="ka-facts xw-facts">
            {#if !exchange}
              <div>
                <dt>{t("export.exportWorkspace.categoryContent")}</dt>
                <dd class="ka-between">
                  <span
                    >{countLabel(selected.length, "chapter")} · {countLabel(
                      sceneCount,
                      "scene"
                    )}</span
                  ><button
                    type="button"
                    class="ka-button ka-button--ghost"
                    aria-label={t("export.exportWorkspace.changeContent")}
                    onclick={() => (section = "content")}
                    >{t("export.exportWorkspace.change")}</button
                  >
                </dd>
              </div>
              {#if !plain}
                <div>
                  <dt>{t("export.exportWorkspace.categoryText")}</dt>
                  <dd class="ka-between">
                    <span>{draft.font} · {draft.fontSize} pt</span><button
                      type="button"
                      class="ka-button ka-button--ghost"
                      aria-label={t("export.exportWorkspace.changeTextPage")}
                      onclick={() => (section = "text")}
                      >{t("export.exportWorkspace.change")}</button
                    >
                  </dd>
                </div>
              {/if}
              <div>
                <dt>{t("export.exportWorkspace.categoryHeadings")}</dt>
                <dd class="ka-between">
                  <span>{draft.separator || t("export.exportWorkspace.blankLine")}</span><button
                    type="button"
                    class="ka-button ka-button--ghost"
                    aria-label={t("export.exportWorkspace.changeHeadings")}
                    onclick={() => (section = "headings")}
                    >{t("export.exportWorkspace.change")}</button
                  >
                </dd>
              </div>
            {:else}
              <div>
                <dt>{t("export.exportWorkspace.categoryContent")}</dt>
                <dd class="ka-between">
                  <span>{t("export.exportWorkspace.wholeProject")}</span><button
                    type="button"
                    class="ka-button ka-button--ghost"
                    aria-label={t("export.exportWorkspace.changeContent")}
                    onclick={() => (section = "files")}>{t("export.exportWorkspace.change")}</button
                  >
                </dd>
              </div>
            {/if}
            <div>
              <dt>{t("export.exportWorkspace.outputFact")}</dt>
              <dd class="ka-between">
                <span>{formatLabels[draft.format]}</span><button
                  type="button"
                  class="ka-button ka-button--ghost"
                  aria-label={t("export.exportWorkspace.changeOutput")}
                  onclick={() => (section = "files")}>{t("export.exportWorkspace.change")}</button
                >
              </dd>
            </div>
          </dl>
          <p class="ka-help">
            {exchange
              ? t("export.exportWorkspace.overviewHelpExchange")
              : t("export.exportWorkspace.overviewHelpApprox")}
          </p>
        </div>
      {:else if section === "content"}
        <div class="xw-group">
          <div class="ka-field od-field">
            <label for="export-scope">{t("export.exportWorkspace.includeInExport")}</label><select
              id="export-scope"
              bind:value={draft.selection}
              onchange={() => {
                draft.sceneId = null;
                if (draft.selection === "selected" && !draft.chapterIds.length)
                  draft.chapterIds = chapters.map((c) => c.id);
              }}
              ><option value="all">{t("export.exportWorkspace.scopeAll")}</option><option
                value="selected">{t("export.exportWorkspace.scopeSelected")}</option
              ></select
            >
          </div>
          {#if contextOverride}<div class="ka-notice od-row-top">
              <Info class="w-5 h-5" aria-hidden="true" />
              <div class="od-field od-fill">
                <p>
                  {t("export.exportWorkspace.contextNotice", { scope: scopeLabel })}
                </p>
              </div>
            </div>{/if}
          {#if draft.sceneId}<div class="ka-notice od-row-top">
              <Info class="w-5 h-5" aria-hidden="true" />
              <div class="od-field od-fill xw-notice-body">
                <p>{t("export.exportWorkspace.onlySelectedScene")}</p>
                <button
                  type="button"
                  class="ka-button ka-button--ghost"
                  onclick={() => (draft.sceneId = null)}
                  >{t("export.exportWorkspace.includeRestOfChapter")}</button
                >
              </div>
            </div>{/if}
          <div
            class="chapter-list"
            role="group"
            aria-label={t("export.exportWorkspace.chapterSelection")}
          >
            {#each chapters as chapter (chapter.id)}<label class="ka-check xw-check chapter-row"
                ><input
                  type="checkbox"
                  checked={draft.selection === "all" || draft.chapterIds.includes(chapter.id)}
                  disabled={draft.selection === "all"}
                  onchange={(event) => selectChapter(chapter.id, event.currentTarget.checked)}
                /><span class="xw-check-text"
                  ><span>{chapter.title}</span><small
                    >{countLabel(chapter.scenes.length, "scene")}{chapter.part
                      ? ` · ${chapter.part}`
                      : ""}</small
                  ></span
                ></label
              >{/each}
          </div>
        </div>
        <div class="xw-group">
          <h4 class="xw-label">{t("export.exportWorkspace.alongsideProse")}</h4>
          <div class="xw-checks">
            <label class="ka-check xw-check"
              ><input
                id="export-scene-titles"
                type="checkbox"
                bind:checked={draft.sceneTitles}
              /><span class="xw-check-text"
                ><span>{t("export.exportWorkspace.sceneTitles")}</span><small
                  >{t("export.exportWorkspace.sceneTitlesHelp")}</small
                ></span
              ></label
            >
            <label class="ka-check xw-check"
              ><input type="checkbox" bind:checked={draft.synopses} /><span class="xw-check-text"
                ><span>{t("export.exportWorkspace.synopses")}</span><small
                  >{t("export.exportWorkspace.synopsesHelp")}</small
                ></span
              ></label
            >
            <label class="ka-check xw-check"
              ><input id="export-beats" type="checkbox" bind:checked={draft.beatHeadings} /><span
                class="xw-check-text"
                ><span>{t("export.exportWorkspace.beatHeadings")}</span><small
                  >{t("export.exportWorkspace.beatHeadingsHelp")}</small
                ></span
              ></label
            >
          </div>
          <p class="ka-help">
            {t("export.exportWorkspace.contentExclusions")}
          </p>
        </div>
      {:else if section === "headings"}
        <div class="xw-group">
          <div class="ka-field od-field">
            <label for="export-heading">{t("export.exportWorkspace.chapterHeading")}</label><select
              id="export-heading"
              bind:value={draft.chapterHeading}
              ><option value="number_title">{t("export.exportWorkspace.headingNumberTitle")}</option
              ><option value="number">{t("export.exportWorkspace.headingNumber")}</option><option
                value="title">{t("export.exportWorkspace.headingTitle")}</option
              ><option value="none">{t("export.exportWorkspace.headingNone")}</option></select
            >
            <p class="ka-help">
              {draft.chapterHeading === "number_title"
                ? t("export.exportWorkspace.headingExampleNumberTitle")
                : draft.chapterHeading === "number"
                  ? t("export.exportWorkspace.headingExampleNumber")
                  : draft.chapterHeading === "title"
                    ? t("export.exportWorkspace.headingExampleTitle")
                    : t("export.exportWorkspace.headingExampleNone")}
            </p>
          </div>
          {#if draft.chapterHeading.startsWith("number")}<div class="xw-pair">
              <div class="ka-field od-field">
                <label for="export-number-style">{t("export.exportWorkspace.numberStyle")}</label
                ><select id="export-number-style" bind:value={draft.numberStyle}
                  ><option value="arabic">1, 2, 3</option><option value="roman">I, II, III</option
                  ></select
                >
              </div>
              <div class="ka-field od-field">
                <label for="export-number-start">{t("export.exportWorkspace.startAt")}</label><input
                  id="export-number-start"
                  type="number"
                  min="1"
                  max="999"
                  bind:value={draft.startNumber}
                />
              </div>
            </div>{/if}
        </div>
        <div class="xw-group">
          <div class="ka-field od-field">
            <label for="export-separator">{t("export.exportWorkspace.betweenScenes")}</label><input
              id="export-separator"
              bind:value={draft.separator}
              maxlength="80"
              aria-describedby="export-separator-help"
            />
            <div
              class="xw-chips"
              role="group"
              aria-label={t("export.exportWorkspace.sceneBreakMarkers")}
            >
              {#each ["#", "* * *", "⁂", ""] as marker (marker)}<button
                  type="button"
                  class="ka-tag"
                  aria-pressed={draft.separator === marker}
                  onclick={() => (draft.separator = marker)}
                  >{marker || t("export.exportWorkspace.blankLine")}</button
                >{/each}
            </div>
            <p class="ka-help" id="export-separator-help">
              {t("export.exportWorkspace.separatorHelp")}
            </p>
          </div>
          <div class="xw-checks">
            <label class="ka-check xw-check"
              ><input type="checkbox" bind:checked={draft.partTitles} /><span class="xw-check-text"
                ><span>{t("export.exportWorkspace.includePartTitles")}</span><small
                  >{t("export.exportWorkspace.includePartTitlesHelp")}</small
                ></span
              ></label
            >
            {#if draft.format === "docx"}<label class="ka-check xw-check"
                ><input
                  id="export-chapter-breaks"
                  type="checkbox"
                  bind:checked={draft.chapterBreaks}
                /><span class="xw-check-text"
                  ><span>{t("export.exportWorkspace.chapterBreaks")}</span><small
                    >{t("export.exportWorkspace.chapterBreaksHelp")}</small
                  ></span
                ></label
              >{/if}
          </div>
        </div>
      {:else if section === "text"}
        <div class="xw-group">
          <div class="xw-pair">
            <div class="ka-field od-field">
              <label for="export-font">{t("export.exportWorkspace.bodyFont")}</label><select
                id="export-font"
                bind:value={draft.font}
                >{#each ["Times New Roman", "Courier New", "Georgia", "Arial"] as font (font)}<option
                    >{font}</option
                  >{/each}</select
              >
            </div>
            <div class="ka-field od-field">
              <label for="export-font-size">{t("export.exportWorkspace.sizePt")}</label><input
                id="export-font-size"
                type="number"
                min="8"
                max="24"
                bind:value={draft.fontSize}
              />
            </div>
          </div>
          <div class="ka-field od-field">
            <label for="export-spacing">{t("export.exportWorkspace.lineSpacing")}</label><select
              id="export-spacing"
              bind:value={draft.lineSpacing}
              ><option value={1}>{t("export.exportWorkspace.spacingSingle")}</option><option
                value={1.5}>{t("export.exportWorkspace.spacingOneHalf")}</option
              ><option value={1.6}>{t("export.exportWorkspace.spacingComfortable")}</option><option
                value={2}>{t("export.exportWorkspace.spacingDouble")}</option
              ><option value={2.5}>{t("export.exportWorkspace.spacingTwoHalf")}</option></select
            >
          </div>
          <div class="xw-pair">
            <div class="ka-field od-field">
              <label for="export-indent">{t("export.exportWorkspace.firstLineIndent")}</label><input
                id="export-indent"
                type="number"
                min="0"
                max="1"
                step="0.05"
                bind:value={draft.indent}
              />
            </div>
            <div class="ka-field od-field">
              <label for="export-paragraph-spacing"
                >{t("export.exportWorkspace.paragraphSpacing")}</label
              ><input
                id="export-paragraph-spacing"
                type="number"
                min="0"
                max="36"
                step="1"
                bind:value={draft.paragraphSpacing}
              />
            </div>
          </div>
          <label class="ka-check"
            ><input type="checkbox" bind:checked={draft.firstParagraphFlush} />{t(
              "export.exportWorkspace.firstParagraphFlush"
            )}</label
          >
          <details class="xw-details">
            <summary
              ><span>{t("export.exportWorkspace.moreTextOptions")}</span><span class="xw-meta"
                >{draft.alignment === "left"
                  ? t("export.exportWorkspace.leftAligned")
                  : t("export.exportWorkspace.justified")}</span
              ></summary
            >
            <div class="ka-field od-field">
              <label for="export-alignment">{t("export.exportWorkspace.bodyAlignment")}</label
              ><select id="export-alignment" bind:value={draft.alignment}
                ><option value="left">{t("export.exportWorkspace.leftAligned")}</option><option
                  value="justify">{t("export.exportWorkspace.justified")}</option
                ></select
              >
            </div>
          </details>
        </div>
        {#if draft.format === "docx"}<div class="xw-group">
            <h4 class="xw-label">{t("export.exportWorkspace.pageLayout")}</h4>
            <div class="xw-pair">
              <div class="ka-field od-field">
                <label for="export-paper">{t("export.exportWorkspace.paperSize")}</label><select
                  id="export-paper"
                  bind:value={draft.paper}
                  ><option value="letter">US Letter</option><option value="a4">A4</option></select
                >
              </div>
              <div class="ka-field od-field">
                <label for="export-margin">{t("export.exportWorkspace.marginsIn")}</label><input
                  id="export-margin"
                  type="number"
                  min="0.25"
                  max="2"
                  step="0.25"
                  bind:value={draft.margin}
                />
              </div>
            </div>
            <p class="ka-help">
              {t("export.exportWorkspace.wordLayoutHelp")}
            </p>
          </div>{:else}<p class="ka-help xw-note">
            {t("export.exportWorkspace.reflowableHelp")}
          </p>{/if}
        {#if draft.format === "html" && !draft.styled}<div class="ka-notice od-row-top xw-note">
            <Info class="w-5 h-5" aria-hidden="true" />
            <div class="od-field od-fill xw-notice-body">
              <p>{t("export.exportWorkspace.htmlStylingOff")}</p>
              <button
                type="button"
                class="ka-button ka-button--ghost"
                onclick={() => (draft.styled = true)}
                >{t("export.exportWorkspace.enableStyling")}</button
              >
            </div>
          </div>{/if}
      {:else if section === "details"}
        <div class="xw-group">
          <div class="ka-field od-field">
            <label for="export-title">{t("export.exportWorkspace.bookTitle")}</label><input
              id="export-title"
              bind:value={draft.title}
            />
          </div>
          <div class="ka-field od-field">
            <label for="export-author">{t("export.exportWorkspace.authorPenName")}</label><input
              id="export-author"
              bind:value={draft.author}
              placeholder={t("export.exportWorkspace.authorPlaceholder")}
            />
          </div>
          <label class="ka-check xw-check"
            ><input type="checkbox" bind:checked={draft.titlePage} /><span class="xw-check-text"
              ><span
                >{plain
                  ? t("export.exportWorkspace.includeTitleBlock")
                  : t("export.exportWorkspace.includeTitlePage")}</span
              ><small>{t("export.exportWorkspace.titlePageHint")}</small></span
            ></label
          >
          {#if draft.titlePage}<div class="ka-field od-field">
              <label for="export-subtitle"
                >{t("export.exportWorkspace.subtitle")}
                <span class="ka-optional">{t("export.exportWorkspace.optional")}</span></label
              ><input id="export-subtitle" bind:value={draft.subtitle} />
            </div>
            <div class="ka-field od-field">
              <label for="export-word-count">{t("export.exportWorkspace.titlePageWordCount")}</label
              ><select id="export-word-count" bind:value={draft.wordCount}
                ><option value="rounded">{t("export.exportWorkspace.wordCountRounded")}</option
                ><option value="exact">{t("export.exportWorkspace.wordCountExact")}</option><option
                  value="none">{t("export.exportWorkspace.wordCountOmit")}</option
                ></select
              >
            </div>{/if}
          {#if draft.format === "docx"}<div class="ka-field od-field">
              <label for="export-header">{t("export.exportWorkspace.runningHeader")}</label><select
                id="export-header"
                bind:value={draft.header}
                ><option value="author_title"
                  >{t("export.exportWorkspace.headerAuthorTitle")}</option
                ><option value="title">{t("export.exportWorkspace.headerTitleOnly")}</option><option
                  value="none">{t("export.exportWorkspace.headerNone")}</option
                ></select
              >
              <p class="ka-help">
                {t("export.exportWorkspace.runningHeaderHelp")}
              </p>
            </div>{/if}
          <label class="ka-check xw-check"
            ><input id="export-contents" type="checkbox" bind:checked={draft.contents} /><span
              class="xw-check-text"
              ><span>{t("export.exportWorkspace.tableOfContents")}</span><small
                >{plain
                  ? t("export.exportWorkspace.contentsPlainHelp")
                  : t("export.exportWorkspace.contentsLinkHelp")}</small
              ></span
            ></label
          >
          {#if draft.format === "epub" || draft.format === "html"}
            <details class="xw-details">
              <summary
                ><span>{t("export.exportWorkspace.language")}</span><span class="xw-meta"
                  >{draft.language}</span
                ></summary
              >
              <div class="ka-field od-field">
                <label for="export-language">{t("export.exportWorkspace.documentLanguage")}</label
                ><select id="export-language" bind:value={draft.language}
                  ><option value="en">{t("export.exportWorkspace.langEnglish")}</option><option
                    value="en-GB">{t("export.exportWorkspace.langEnglishUK")}</option
                  ><option value="fr">{t("export.exportWorkspace.langFrench")}</option><option
                    value="de">{t("export.exportWorkspace.langGerman")}</option
                  ><option value="es">{t("export.exportWorkspace.langSpanish")}</option><option
                    value="pt">{t("export.exportWorkspace.langPortuguese")}</option
                  ></select
                >
              </div>
            </details>
          {/if}
        </div>
        {#if draft.format === "epub"}<div class="xw-group">
            <h4 class="xw-label">{t("export.exportWorkspace.ebookMetadata")}</h4>
            <div class="ka-field od-field">
              <label for="export-description">{t("export.exportWorkspace.description")}</label
              ><textarea id="export-description" rows="3" bind:value={draft.description}></textarea>
            </div>
            <div class="ka-field od-field">
              <label for="export-cover">{t("export.exportWorkspace.coverImage")}</label>
              <div class="xw-location">
                <input
                  id="export-cover"
                  readonly
                  value={draft.coverPath}
                  placeholder={t("export.exportWorkspace.coverPlaceholder")}
                  aria-describedby="export-cover-help"
                />
                <button type="button" class="ka-button ka-button--secondary" onclick={chooseCover}
                  >{t("export.exportWorkspace.chooseCover")}</button
                >{#if draft.coverPath}<button
                    type="button"
                    class="ka-button ka-button--ghost"
                    onclick={() => (draft.coverPath = "")}
                    >{t("export.exportWorkspace.remove")}</button
                  >{/if}
              </div>
              <p class="ka-help" id="export-cover-help">
                {t("export.exportWorkspace.coverHelp")}
              </p>
            </div>
          </div>
        {/if}
      {:else if section === "files"}
        <div class="xw-group">
          <div class="ka-field od-field">
            <label for="export-filename">{t("export.exportWorkspace.filenamePattern")}</label><input
              id="export-filename"
              bind:value={draft.fileName}
              aria-describedby="export-filename-sample"
            />
            <div
              class="xw-chips"
              role="group"
              aria-label={t("export.exportWorkspace.filenameTokens")}
            >
              {#each ["{title}", "{profile}", "{date}"] as token (token)}<button
                  type="button"
                  class="ka-tag"
                  onclick={() => (draft.fileName += token)}>+ {token}</button
                >{/each}
            </div>
            <p class="ka-help filename-sample" id="export-filename-sample">{fileName}</p>
          </div>
          {#if draft.format === "html"}<div class="ka-field od-field">
              <label for="export-html-mode">{t("export.exportWorkspace.htmlStructure")}</label
              ><select id="export-html-mode" bind:value={draft.htmlMode}
                ><option value="document">{t("export.exportWorkspace.htmlDocument")}</option><option
                  value="fragment">{t("export.exportWorkspace.htmlFragment")}</option
                ></select
              >
              <p class="ka-help">{t("export.exportWorkspace.htmlSavedNote")}</p>
            </div>
            <div class="ka-field od-field">
              <label for="export-html-heading"
                >{t("export.exportWorkspace.htmlHeadingElement")}</label
              ><select id="export-html-heading" bind:value={draft.headingLevel}
                ><option value="h1">{t("export.exportWorkspace.heading1Option")}</option><option
                  value="h2">{t("export.exportWorkspace.heading2Option")}</option
                ></select
              >
            </div>
            <label class="ka-check xw-check"
              ><input type="checkbox" bind:checked={draft.styled} /><span class="xw-check-text"
                ><span>{t("export.exportWorkspace.builtInStyling")}</span><small
                  >{t("export.exportWorkspace.builtInStylingHelp")}</small
                ></span
              ></label
            >{/if}
          {#if draft.format === "novelwriter"}
            <div class="xw-checks">
              <label class="ka-check xw-check"
                ><input type="checkbox" bind:checked={draft.includeNotes} /><span
                  class="xw-check-text"
                  ><span>{t("export.exportWorkspace.referenceNotes")}</span><small
                    >{t("export.exportWorkspace.referenceNotesHelp")}</small
                  ></span
                ></label
              >
              <label class="ka-check xw-check"
                ><input type="checkbox" bind:checked={draft.includeBeatComments} /><span
                  class="xw-check-text"
                  ><span>{t("export.exportWorkspace.beatComments")}</span><small
                    >{t("export.exportWorkspace.beatCommentsHelp")}</small
                  ></span
                ></label
              >
            </div>
          {:else if draft.format === "treatment"}
            <div class="ka-field od-field">
              <label for="export-treatment-level"
                >{t("export.exportWorkspace.treatmentDetail")}</label
              ><select id="export-treatment-level" bind:value={draft.treatmentLevel}
                ><option value="one_page">{t("export.exportWorkspace.treatmentOnePage")}</option
                ><option value="five_page">{t("export.exportWorkspace.treatmentFivePage")}</option
                ><option value="full">{t("export.exportWorkspace.treatmentFull")}</option></select
              >
              <p class="ka-help">
                {t("export.exportWorkspace.treatmentDetailHelp")}
              </p>
            </div>
            <div class="ka-field od-field">
              <label for="export-treatment-format"
                >{t("export.exportWorkspace.treatmentFile")}</label
              ><select id="export-treatment-format" bind:value={draft.treatmentFormat}
                ><option value="docx">{t("export.exportWorkspace.treatmentDocx")}</option><option
                  value="txt">{t("export.exportWorkspace.treatmentTxt")}</option
                ></select
              >
            </div>
          {:else if draft.format === "scrivener"}<p class="ka-help">
              {t("export.exportWorkspace.scrivenerHelp")}
            </p>
          {:else if draft.format === "longform"}<p class="ka-help">
              {t("export.exportWorkspace.longformHelp")}
            </p>{/if}
        </div>
        <div class="xw-group">
          <div class="xw-file">
            <FileText class="w-5 h-5" aria-hidden="true" />
            <div class="od-field od-fill">
              <strong>{formatLabels[draft.format]}</strong><span class="xw-meta"
                >{exchange
                  ? t("export.exportWorkspace.fileWholeProject")
                  : `${countLabel(selected.length, "chapter")} · ${countLabel(sceneCount, "scene")}`}</span
              >
            </div>
            <span class="xw-file-type"
              >{["longform", "novelwriter"].includes(draft.format)
                ? t("export.exportWorkspace.fileTypeFolder")
                : `.${outputExtension(draft)}`}</span
            >
          </div>
          <p class="ka-help">
            {t("export.exportWorkspace.destinationHelp")}
          </p>
          {#if draft.format === "markdown"}<p class="ka-help">
              {t("export.exportWorkspace.markdownHelp")}
            </p>{/if}
        </div>
      {/if}
    </section>

    <section
      class="xw-preview"
      class:mobile-hidden={narrowPane === "settings"}
      aria-label={t("export.exportWorkspace.previewAria")}
    >
      {#if exchange}<div class="ka-empty od-stack empty-preview">
          <FileDown class="w-7 h-7" aria-hidden="true" />
          <h4>{formatLabels[draft.format]}</h4>
          <p>
            {draft.format === "treatment"
              ? t("export.exportWorkspace.treatmentPreview")
              : t("export.exportWorkspace.exchangePreview")}
          </p>
          <p class="filename-sample">{fileName}</p>
          <p>
            {draft.format === "scrivener"
              ? t("export.exportWorkspace.scrivenerPreview")
              : draft.format === "longform"
                ? t("export.exportWorkspace.longformPreview")
                : draft.format === "novelwriter"
                  ? t("export.exportWorkspace.novelwriterPreview")
                  : t("export.exportWorkspace.treatmentPreviewDetail")}
          </p>
        </div>{:else}
        <header class="xw-preview-head">
          <div class="xw-preview-title">
            <span class="xw-label">{t("export.exportWorkspace.livePreview")}</span>
            <span class="ka-badge ka-badge--success">{t("export.exportWorkspace.liveBadge")}</span>
            {#if draft.format === "html"}<div
                class="ka-segment-track preview-tabs"
                role="group"
                aria-label={t("export.exportWorkspace.previewView")}
              >
                <button
                  type="button"
                  class="ka-segment"
                  class:ka-selected={previewMode === "rendered"}
                  aria-pressed={previewMode === "rendered"}
                  onclick={() => (previewMode = "rendered")}
                  >{t("export.exportWorkspace.previewRead")}</button
                ><button
                  type="button"
                  class="ka-segment"
                  class:ka-selected={previewMode === "source"}
                  aria-pressed={previewMode === "source"}
                  onclick={() => (previewMode = "source")}
                  >{t("export.exportWorkspace.previewHtml")}</button
                >
              </div>
            {/if}
          </div>
          <div class="xw-preview-controls">
            {#if plain}<p class="xw-meta od-fill">
                {t("export.exportWorkspace.wholeSelectionSingleFile")}
              </p>{:else}<div class="ka-field od-field od-fill">
                <label class="ka-sr" for="export-preview-chapter"
                  >{t("export.exportWorkspace.previewChapter")}</label
                ><select
                  id="export-preview-chapter"
                  value={actualPreviewChapter}
                  onchange={(event) => (previewChapter = event.currentTarget.value)}
                  disabled={!selected.length}
                  ><option value=""
                    >{selected.length
                      ? t("export.exportWorkspace.wholeSelection")
                      : t("export.exportWorkspace.noChaptersSelected")}</option
                  >{#each selected as chapter (chapter.id)}<option value={chapter.id}
                      >{chapter.title}</option
                    >{/each}</select
                >
              </div>{/if}<button
              type="button"
              class="ka-button ka-button--ghost ka-icon-button"
              aria-label={t("export.exportWorkspace.refreshManuscript")}
              title={t("export.exportWorkspace.refreshManuscript")}
              disabled={loading}
              aria-busy={loading || undefined}
              onclick={() => loadManuscript()}
              ><RefreshCw
                class={loading ? "w-5 h-5 animate-spin" : "w-5 h-5"}
                aria-hidden="true"
              /></button
            >
          </div>
        </header>
        <div class="xw-desk">
          {#if loading}<div class="ka-empty od-stack empty-preview" role="status">
              <Loader2 class="w-7 h-7 animate-spin" aria-hidden="true" />
              <h4>{t("export.exportWorkspace.loadingManuscript")}</h4>
            </div>{:else if !selected.length}<div class="ka-empty od-stack empty-preview">
              <BookOpen class="w-7 h-7" aria-hidden="true" />
              <h4>{t("export.exportWorkspace.nothingSelected")}</h4>
              <p>{t("export.exportWorkspace.nothingSelectedHelp")}</p>
              <button
                type="button"
                class="ka-button ka-button--secondary"
                onclick={() => {
                  section = "content";
                  narrowPane = "settings";
                }}
                >{t("export.exportWorkspace.chooseContent")}
                <ArrowRight class="w-5 h-5" aria-hidden="true" /></button
              >
            </div>{:else if (draft.format === "html" && previewMode === "source") || plain}<textarea
              class="source-preview"
              readonly
              aria-label={plain
                ? t("export.exportWorkspace.generatedTextAria")
                : t("export.exportWorkspace.generatedHtmlAria")}
              value={sourceText}
            ></textarea>{:else}<iframe
              title={t("export.exportWorkspace.layoutPreviewTitle")}
              sandbox=""
              srcdoc={previewDocument}
              data-ready={previewLoaded === previewDocument}
              onload={(event) => (previewLoaded = event.currentTarget.getAttribute("srcdoc") ?? "")}
            ></iframe>{/if}
        </div>
        <footer class="xw-preview-foot">
          <span
            >{draft.format === "html"
              ? t("export.exportWorkspace.htmlOutput")
              : plain
                ? t("export.exportWorkspace.textOutput")
                : t("export.exportWorkspace.layoutApproximation")}</span
          ><span
            >{t("export.exportWorkspace.previewSummary", {
              words: wordCount.toLocaleString(),
              scenes: countLabel(sceneCount, "scene"),
            })}</span
          >
        </footer>
      {/if}
    </section>
  </div>

  {#if error || storageError || missingSelection || message || unsupported || validationError}
    <div class="xw-feedback">
      {#if error || storageError}<div class="ka-notice ka-notice--error od-row-top" role="alert">
          <TriangleAlert class="w-5 h-5" aria-hidden="true" />
          <div class="od-field od-fill xw-notice-body">
            <p>{error || storageError}</p>
            {#if !loading && error.startsWith(t("export.exportWorkspace.loadFailed"))}<button
                type="button"
                class="ka-button ka-button--ghost"
                onclick={() => loadManuscript()}>{t("common.retry")}</button
              >{/if}
          </div>
        </div>{/if}
      {#if missingSelection}<div class="ka-notice ka-notice--error od-row-top" role="alert">
          <TriangleAlert class="w-5 h-5" aria-hidden="true" />
          <div class="od-field od-fill">
            <p>{t("export.exportWorkspace.missingSelection")}</p>
          </div>
        </div>{/if}
      {#if message}<div class="ka-notice od-row-top" role="status">
          <Info class="w-5 h-5" aria-hidden="true" />
          <div class="od-field od-fill xw-notice-body">
            <p>{message}</p>
            {#if savedPath}<button
                type="button"
                class="ka-button ka-button--ghost"
                onclick={openSaved}
                >{t("export.exportWorkspace.showInFolder")}
                <ArrowRight class="w-5 h-5" aria-hidden="true" /></button
              >{/if}
          </div>
        </div>{/if}
      {#if unsupported}<div class="ka-notice ka-notice--error od-row-top" role="alert">
          <TriangleAlert class="w-5 h-5" aria-hidden="true" />
          <div class="od-field od-fill"><p>{unsupported}</p></div>
        </div>{/if}
      {#if validationError}<div class="ka-notice ka-notice--error od-row-top" role="alert">
          <TriangleAlert class="w-5 h-5" aria-hidden="true" />
          <div class="od-field od-fill"><p>{validationError}</p></div>
        </div>{/if}
    </div>
  {/if}
  <footer class="xw-foot">
    <div class="ka-help xw-foot-note">
      <div>
        {exchange
          ? t("export.exportWorkspace.wholeProject")
          : t("export.exportWorkspace.footOriginSaved")} → {formatLabels[draft.format]}
      </div>
      <div>{t("export.exportWorkspace.footNote")}</div>
    </div>
    <div class="ka-row xw-foot-actions">
      {#if dirty}<button
          type="button"
          class="ka-button ka-button--ghost"
          disabled={saving}
          onclick={() => {
            if (savedProfile) draft = window.structuredClone($state.snapshot(savedProfile));
            contextSelection = null;
          }}>{t("export.exportWorkspace.revert")}</button
        >{/if}<button
        type="button"
        class="ka-button ka-button--secondary"
        disabled={saving || storageBlocked || !!validationError}
        onclick={() => saveProfile()}>{t("export.exportWorkspace.saveProfile")}</button
      ><button
        type="button"
        class="ka-button"
        disabled={saving ||
          loading ||
          !documentLoaded ||
          (!exchange && !sceneCount) ||
          !!unsupported ||
          missingSelection ||
          !!validationError}
        aria-busy={saving || undefined}
        onclick={saveExport}
        >{#if saving}<Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />{t(
            "export.exportWorkspace.exporting"
          )}{:else}<Download class="w-5 h-5" aria-hidden="true" />{draft.format === "longform" ||
          draft.format === "novelwriter"
            ? t("export.exportWorkspace.exportProjectButton")
            : t("export.exportWorkspace.exportButton", {
                format:
                  draft.format === "treatment"
                    ? draft.treatmentFormat.toUpperCase()
                    : outputExtension(draft).toUpperCase(),
              })}{/if}</button
      >
    </div>
  </footer>
</dialog>

<style>
  /* Full-window view: the native modal dialog fills the app window and sits in
     the top layer, so the workspace underneath is inert. */
  .export-workspace {
    position: fixed;
    inset: 0;
    z-index: var(--z-modal);
    width: 100%;
    height: 100%;
    max-width: none;
    max-height: none;
    margin: 0;
    padding: 0;
    border: 0;
    border-radius: 0;
    background: var(--color-bg);
    color: var(--color-text);
    overflow: hidden;
    font: var(--text-ui) / 1.5 var(--font-ui);
  }
  .export-workspace[open] {
    display: flex;
    flex-direction: column;
  }
  .export-workspace::backdrop {
    background: var(--color-overlay-scrim);
  }

  .xw-meta {
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .xw-label {
    margin: 0;
    padding: 0;
    font: 500 var(--text-ui) / 1.4 var(--font-ui);
    letter-spacing: normal;
    color: var(--color-text);
  }

  /* App bar */
  .xw-appbar {
    flex: none;
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    min-height: 72px;
    padding: var(--space-xs) var(--space-m) var(--space-xs) var(--space-m);
    border-bottom: var(--border-hair);
    background: var(--color-bg);
  }
  .xw-appbar h3 {
    margin: 0;
    font: 550 var(--text-h3) / var(--leading-tight) var(--font-display);
    letter-spacing: var(--tracking-tight);
    color: var(--color-text);
  }
  .xw-appbar-actions {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
  }

  /* Profile tools */
  .xw-tools {
    flex: none;
    display: flex;
    align-items: flex-end;
    flex-wrap: wrap;
    gap: var(--space-s) var(--space-m);
    padding: var(--space-s) var(--space-m);
    border-bottom: var(--border-hair);
  }
  .xw-picker {
    width: auto;
  }
  .xw-picker select {
    min-width: 260px;
  }
  .xw-status {
    margin-left: auto;
    display: grid;
    justify-items: end;
    gap: var(--space-3xs);
    text-align: right;
  }
  .xw-ok {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2xs);
    font: 500 var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-success);
  }

  /* Narrow-window view switch */
  .xw-switch {
    display: none;
  }

  /* Three panes */
  .xw-body {
    flex: 1 1 auto;
    min-height: 0;
    display: grid;
    grid-template-columns: 280px minmax(0, 1fr) minmax(420px, 42%);
  }

  .xw-nav {
    display: flex;
    flex-direction: column;
    gap: var(--space-s);
    min-width: 0;
    padding: var(--space-s);
    border-right: var(--border-hair);
    overflow: auto;
  }
  .xw-search {
    display: flex;
    align-items: center;
    gap: var(--space-3xs);
  }
  .xw-nav .ka-tree-list {
    gap: var(--space-3xs);
  }
  .xw-nav .ka-tree button {
    align-items: flex-start;
  }
  .xw-nav .ka-tree small {
    display: block;
  }
  .xw-nav-note {
    margin-top: auto;
  }
  .xw-results {
    display: grid;
    margin: 0;
    padding: 0;
    list-style: none;
    border-top: var(--border-hair);
  }
  .xw-results > li {
    border-bottom: var(--border-hair);
  }
  .xw-row-button {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    width: 100%;
    min-height: var(--control-target);
    padding: var(--space-2xs) var(--space-2xs) var(--space-2xs) 0;
    border: 0;
    background: transparent;
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
    font: var(--text-ui) / 1.5 var(--font-ui);
  }
  @media (hover: hover) {
    .xw-row-button:hover {
      background: var(--color-surface-sunken);
    }
  }
  .xw-row-button:focus-visible {
    outline: 2px solid var(--color-accent-text);
    outline-offset: -2px;
  }
  .xw-empty {
    padding: var(--space-s) 0;
  }
  .ka-empty h4 {
    margin: 0;
    font: 550 var(--text-h3) / var(--leading-tight) var(--font-display);
    letter-spacing: var(--tracking-tight);
    color: var(--color-text);
  }

  /* Settings pane */
  .xw-pane {
    min-width: 0;
    overflow: auto;
    padding: var(--space-l) var(--space-xl) var(--space-xl);
  }
  .xw-pane-head {
    display: grid;
    gap: var(--space-3xs);
    margin-bottom: var(--space-l);
  }
  .xw-pane-head h3 {
    margin: 0;
    font: 550 var(--text-h2) / var(--leading-tight) var(--font-display);
    letter-spacing: var(--tracking-tight);
    color: var(--color-text);
  }
  .xw-group {
    display: grid;
    gap: var(--space-s);
    min-width: 0;
    padding-top: var(--space-m);
    margin-top: var(--space-m);
    border-top: var(--border-hair);
  }
  .xw-pane-head + .xw-group {
    margin-top: 0;
    padding-top: 0;
    border-top: 0;
  }
  .xw-note {
    margin-top: var(--space-m);
  }
  .xw-pair {
    display: grid;
    grid-template-columns: minmax(0, 3fr) minmax(0, 2fr);
    gap: var(--space-s);
  }
  .xw-checks {
    display: grid;
  }
  .xw-check {
    align-items: flex-start;
    padding-block: var(--space-2xs);
    cursor: pointer;
  }
  .xw-check input {
    margin-top: 2px;
  }
  .xw-check-text {
    display: grid;
    gap: 2px;
    min-width: 0;
  }
  .xw-check-text small {
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .chapter-list {
    display: grid;
    max-height: 264px;
    overflow-y: auto;
    border-block: var(--border-hair);
  }
  .chapter-list .chapter-row + .chapter-row {
    border-top: var(--border-hair);
  }
  .xw-choice-list {
    display: grid;
    gap: 0;
    min-width: 0;
    margin: 0;
    padding: 0;
    border: 0;
  }
  .xw-choice-list legend {
    margin-bottom: var(--space-3xs);
    padding: 0;
  }
  .xw-choice {
    align-items: flex-start;
    padding-block: var(--space-2xs);
    border-bottom: var(--border-hair);
  }
  .xw-choice:last-child {
    border-bottom: 0;
  }
  .xw-choice input {
    margin-top: 2px;
  }
  .xw-choice > .od-field {
    gap: 0;
  }
  .xw-choice label {
    min-height: 0;
  }
  .xw-choice .ka-help {
    margin: 0;
  }
  .xw-facts {
    column-gap: 0;
  }
  .xw-facts dt {
    display: flex;
    align-items: center;
    padding-right: var(--space-s);
  }
  .xw-facts dt,
  .xw-facts dd {
    padding-block: var(--space-3xs);
    border-bottom: var(--border-hair);
  }
  .xw-facts dd {
    flex-wrap: nowrap;
  }
  .xw-notice-body {
    justify-items: start;
  }
  /* Optically align a notice's ghost action with its text. */
  .xw-notice-body > .ka-button--ghost {
    margin-left: calc(-1 * var(--space-s));
  }
  .xw-chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2xs);
  }
  .xw-chips .ka-tag[aria-pressed="true"] {
    color: var(--color-on-accent);
    background: var(--color-accent-text);
    border-color: var(--color-accent-text);
  }
  .xw-chips .ka-tag:focus-visible {
    outline: 2px solid var(--color-accent-text);
    outline-offset: 3px;
  }
  .xw-details {
    border-block: var(--border-hair);
  }
  .xw-details summary {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-s);
    min-height: var(--control-target);
    cursor: pointer;
    font: var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .xw-details summary:focus-visible {
    outline: 2px solid var(--color-accent-text);
    outline-offset: 3px;
  }
  .xw-details > .ka-field {
    padding-bottom: var(--space-s);
  }
  .xw-location {
    display: flex;
    gap: var(--space-2xs);
  }
  .xw-location input {
    flex: 1;
    min-width: 0;
  }
  .xw-file {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding-block: var(--space-xs);
    border-block: var(--border-hair);
  }
  .xw-file strong {
    font-weight: 500;
  }
  .xw-file-type,
  .filename-sample {
    font: var(--text-small) / 1.5 var(--font-mono);
    color: var(--color-text-muted);
    overflow-wrap: anywhere;
  }

  /* Preview desk */
  .xw-preview {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    background: var(--color-surface-sunken);
    border-left: var(--border-hair);
  }
  .xw-preview-head {
    flex: none;
    display: grid;
    gap: var(--space-2xs);
    padding: var(--space-s) var(--space-m);
  }
  .xw-preview-title {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    min-height: var(--control-target);
  }
  .preview-tabs {
    margin-left: auto;
    flex-wrap: nowrap;
  }
  .xw-preview-controls {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
  }
  .xw-desk {
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  /* The previewed document is light; a matching scheme keeps WebKit from
     painting an opaque backdrop behind the frame in the dark theme. */
  .xw-desk iframe {
    color-scheme: light;
    flex: 1;
    width: 100%;
    min-height: 0;
    border: 0;
    border-block: var(--border-hair);
    background: transparent;
  }
  /* Generated text reads on the always-light manuscript sheet. */
  .source-preview {
    flex: 1;
    min-height: 0;
    margin: 0 var(--space-m) var(--space-m);
    padding: var(--space-m);
    border: 0;
    border-radius: var(--radius-xs);
    resize: none;
    overflow: auto;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    background: var(--color-prose-bg);
    color: var(--color-prose-text);
    box-shadow: var(--shadow-prose);
    font: var(--text-small) / var(--leading-relaxed) var(--font-mono);
  }
  .empty-preview {
    flex: 1;
    padding: var(--space-l) var(--space-m);
  }
  .empty-preview p {
    font: var(--text-ui) / 1.6 var(--font-ui);
  }
  .xw-preview-foot {
    flex: none;
    display: flex;
    justify-content: space-between;
    gap: var(--space-s);
    padding: var(--space-xs) var(--space-m);
    border-top: var(--border-hair);
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }

  /* Messages and footer */
  .xw-feedback {
    flex: none;
    display: grid;
    gap: var(--space-2xs);
    max-height: 30dvh;
    overflow: auto;
    padding: var(--space-xs) var(--space-m);
    border-top: var(--border-hair);
  }
  .xw-foot {
    flex: none;
    display: flex;
    align-items: center;
    gap: var(--space-s);
    padding: var(--space-s) var(--space-m);
    border-top: var(--border-hair);
    background: var(--color-bg);
  }
  .xw-foot-actions {
    margin-left: auto;
    flex-wrap: nowrap;
  }

  @media (max-width: 1280px) {
    .xw-body {
      grid-template-columns: 232px minmax(0, 1fr) 400px;
    }
    .xw-pane {
      padding: var(--space-m);
    }
  }
  @media (max-width: 900px) {
    .xw-appbar,
    .xw-tools,
    .xw-foot {
      padding-inline: var(--space-s);
    }
    .xw-status,
    .xw-foot-note {
      display: none;
    }
    .xw-picker {
      flex: 1;
    }
    .xw-picker select {
      min-width: 0;
    }
    .xw-switch {
      display: flex;
      flex: none;
      margin: var(--space-2xs) var(--space-s);
    }
    .xw-body {
      grid-template-columns: 232px minmax(0, 1fr);
    }
    .xw-preview {
      grid-column: 1 / -1;
      border-left: 0;
    }
    .xw-body .mobile-hidden {
      display: none;
    }
    .xw-pair {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .export-workspace :global(.animate-spin) {
      animation: none;
    }
  }
</style>
