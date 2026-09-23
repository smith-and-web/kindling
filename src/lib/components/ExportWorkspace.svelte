<script lang="ts">
  import { countLabel } from "../utils/plural";
  import { onMount, tick, untrack } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { openPath } from "@tauri-apps/plugin-opener";
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
    { id: "overview", label: "Overview", description: "A familiar starting point" },
    { id: "content", label: "Content", description: "Choose what goes in" },
    { id: "headings", label: "Headings & breaks", description: "Give the story structure" },
    { id: "text", label: "Text & page", description: "Set the reading rhythm" },
    { id: "details", label: "Book details", description: "Title, author & contents" },
    { id: "files", label: "Files & format", description: "Prepare the handoff" },
  ];
  const presetDescriptions: Record<string, string> = {
    docx: "A clean, familiar submission format",
    epub: "A comfortable copy for your readers",
  };
  const settingsIndex = [
    {
      label: "Included chapters",
      terms: "selection scene manuscript scope",
      section: "content",
      control: "scope",
    },
    {
      label: "Scene titles and synopses",
      terms: "notes summary outline",
      section: "content",
      control: "scene-titles",
    },
    { label: "Beat headings", terms: "outline prompts", section: "content", control: "beats" },
    {
      label: "Chapter headings",
      terms: "title number roman numbering",
      section: "headings",
      control: "heading",
    },
    {
      label: "Scene separator",
      terms: "break marker hash asterisks",
      section: "headings",
      control: "separator",
    },
    {
      label: "Chapter page breaks",
      terms: "new page",
      section: "headings",
      control: "chapter-breaks",
    },
    {
      label: "Font and size",
      terms: "typeface times courier georgia",
      section: "text",
      control: "font",
    },
    {
      label: "Line spacing",
      terms: "double spaced single leading",
      section: "text",
      control: "spacing",
    },
    {
      label: "Paragraph spacing and indent",
      terms: "first line flush",
      section: "text",
      control: "indent",
    },
    {
      label: "Paper and margins",
      terms: "a4 letter gutter page size",
      section: "text",
      control: "margin",
    },
    {
      label: "Title page and pen name",
      terms: "author identity subtitle",
      section: "details",
      control: "title",
    },
    {
      label: "Running header",
      terms: "running head author title",
      section: "details",
      control: "header",
    },
    {
      label: "Table of contents",
      terms: "toc navigation",
      section: "details",
      control: "contents",
    },
    {
      label: "HTML structure and styling",
      terms: "fragment full document css web",
      section: "files",
      control: "html-mode",
    },
    {
      label: "Filename pattern",
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
      ? "novelWriter export supports novel projects only."
      : ""
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

  /** The preview sits on the app's desk: drop the export's own page backdrop
      (the exported file keeps it) so the sheet reads as paper in both themes. */
  /** Keep the selected section in view when the nav scrolls (compact windows). */
  function keepVisible(node: HTMLElement, selected: boolean) {
    const reveal = (on: boolean) => {
      if (on) node.scrollIntoView?.({ block: "nearest" });
    };
    reveal(selected);
    return { update: reveal };
  }

  function deskDocument(html: string): string {
    // The sandboxed frame can't read app tokens, so pass the prose shadow in.
    const probe = document.createElement("span");
    probe.style.boxShadow = "var(--shadow-prose)";
    document.body.appendChild(probe);
    const shadow = getComputedStyle(probe).boxShadow || "none";
    probe.remove();
    const mount = `body{background:transparent;padding:16px 24px 32px}.manuscript{margin:0 auto;box-shadow:${shadow}}`;
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
            message = "An incomplete draft could not be restored. Your saved profile was loaded.";
          }
        }
      }
    } catch (e) {
      storageBlocked = true;
      storageError = `Could not load saved profiles. Their stored data is preserved. ${String(e)}`;
    }
    ready = true;
    void loadManuscript(true);
  });

  $effect(() => {
    if (!ready || storageBlocked) return;
    // Context-menu scope is temporary until explicitly saved or duplicated.
    const journal = contextSelection ? { ...draft, ...contextSelection } : draft;
    const data = JSON.stringify({ version: 2, profiles, activeId: draft.id, draft: journal });
    try {
      localStorage.setItem(profileStorageKey(project.id), data);
    } catch (e) {
      storageError = `Could not retain the draft on this device: ${String(e)}`;
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
      error = `Could not load the saved manuscript: ${String(e)}`;
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
      message = "Give this profile a name first.";
      section = "overview";
      return;
    }
    if (storageBlocked) {
      message = "Profile storage could not be loaded; saving is disabled to preserve it.";
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
      message = `“${draft.name}” saved on this device.`;
    } catch (e) {
      storageError = `Could not save profile: ${String(e)}`;
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
      message = "Copy created. Give it a name for your next writing routine.";
    } catch (e) {
      message = `Could not duplicate profile: ${String(e)}`;
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
    } else message = `“${item.label}” applies to a different output format.`;
  }

  async function saveExport() {
    if (validationError || unsupported) return;
    saving = true;
    error = "";
    savedPath = "";
    const profile = $state.snapshot(draft);
    const filename = exportFilename(profile);
    try {
      let path: string | null;
      if (["longform", "novelwriter"].includes(profile.format)) {
        const parent = await open({
          title: "Choose the parent folder for a new export",
          directory: true,
        });
        path = parent ? `${parent.replace(/[\\/]$/, "")}/${filename}` : null;
      } else {
        path = await save({
          title: `Export ${formatLabels[profile.format]} (choose a new name)`,
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
      message = `${formatLabels[profile.format]} exported. ${isExchange(profile.format) ? "The whole project was used." : "The full selection was included."}`;
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
  async function chooseCover() {
    try {
      const path = await open({
        title: "Choose an ebook cover",
        filters: [{ name: "Cover image", extensions: ["png", "jpg", "jpeg"] }],
      });
      if (path) draft.coverPath = path;
    } catch (e) {
      error = `Could not choose a cover image: ${String(e)}`;
    }
  }

  async function openSaved() {
    try {
      await openPath(savedPath);
    } catch (e) {
      error = `Could not open export: ${String(e)}`;
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
    <h3 id="export-workspace-title">Export workspace</h3>
    <div class="xw-appbar-actions">
      <button type="button" class="ka-button ka-button--ghost" onclick={onClassic} disabled={saving}
        ><ArrowLeft class="w-5 h-5" aria-hidden="true" />Back to export</button
      ><button
        type="button"
        class="ka-button ka-button--ghost ka-icon-button"
        aria-label="Close export workspace"
        title="Close export workspace"
        onclick={onClose}
        disabled={saving}><X class="w-5 h-5" aria-hidden="true" /></button
      >
    </div>
  </header>

  <div class="xw-tools">
    <div class="ka-field od-field xw-picker">
      <label for="export-profile">Export profile</label>
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
      aria-label="Duplicate profile"
      onclick={duplicateProfile}
      disabled={storageBlocked}>Duplicate profile</button
    >
    <div class="ka-field od-field xw-picker">
      <label for="export-format">Output format</label><select
        id="export-format"
        bind:value={draft.format}
        ><option value="docx">Word manuscript</option><option value="epub"
          >Ebook / reader copy</option
        ><option value="html">Web / HTML</option><option value="markdown">Markdown</option><option
          value="txt">Plain text</option
        ><option value="longform">Longform / Obsidian</option><option value="scrivener"
          >Scrivener</option
        ><option value="novelwriter">novelWriter</option><option value="treatment">Treatment</option
        ></select
      >
    </div>
    <div class="xw-status" role="status">
      {#if dirty}<span class="ka-badge ka-badge--warning">Unsaved changes</span>{:else}<span
          class="xw-ok"><Check class="w-5 h-5" aria-hidden="true" />Profile saved</span
        >{/if}
      <span class="xw-meta">Profiles stay on this device</span>
    </div>
  </div>

  <div class="ka-segment-track xw-switch" role="group" aria-label="Workspace view">
    <button
      type="button"
      class="ka-segment"
      class:ka-selected={narrowPane === "settings"}
      aria-pressed={narrowPane === "settings"}
      onclick={() => (narrowPane = "settings")}>Settings</button
    ><button
      type="button"
      class="ka-segment"
      class:ka-selected={narrowPane === "preview"}
      aria-pressed={narrowPane === "preview"}
      onclick={() => (narrowPane = "preview")}>Preview</button
    >
  </div>

  <div class="xw-body">
    <aside class="xw-nav" class:mobile-hidden={narrowPane === "preview"}>
      <div class="xw-search">
        <div class="ka-field od-field od-fill">
          <input
            type="search"
            aria-label="Find an export setting"
            placeholder="Find a setting…"
            bind:value={query}
          />
        </div>
        {#if query}<button
            type="button"
            class="ka-button ka-button--ghost ka-icon-button"
            aria-label="Clear settings search"
            title="Clear settings search"
            onclick={() => (query = "")}><X class="w-5 h-5" aria-hidden="true" /></button
          >{/if}
      </div>
      {#if query.trim()}
        {#if searchResults.length}
          <p class="xw-meta" role="status">{searchResults.length} settings found</p>
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
            <h4>No settings match “{query.trim()}”</h4>
            <p>Try “spacing,” “header,” or “HTML.”</p>
          </div>
        {/if}
      {:else}
        <nav class="ka-tree" aria-label="Export settings">
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
          Make it yours. Your manuscript stays exactly as you wrote it.
        </p>
      {/if}
    </aside>

    <section
      class="settings-panel xw-pane"
      class:mobile-hidden={narrowPane === "preview"}
      aria-label="Export configuration"
    >
      <div class="xw-pane-head">
        <p class="xw-meta">{formatLabels[draft.format]}</p>
        <h3>{visibleCategory.label}</h3>
        <p class="ka-help">{visibleCategory.description}.</p>
      </div>

      {#if section === "overview"}
        <div class="xw-group">
          <div class="ka-field od-field">
            <label for="export-name">Profile name</label><input
              id="export-name"
              bind:value={draft.name}
              maxlength="100"
              aria-invalid={!draft.name.trim() || undefined}
              aria-describedby="export-name-help"
            />
            <p class="ka-help" id="export-name-help">
              Name it for a recipient or a writing routine.
            </p>
          </div>
        </div>
        <div class="xw-group">
          <fieldset class="xw-choice-list">
            <legend class="xw-label">A starting point for every handoff</legend>
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
                    {presetDescriptions[preset.format] ?? "Clean markup for your next destination"}
                  </p>
                </div>
              </div>{/each}
          </fieldset>
        </div>
        <div class="xw-group">
          <h4 class="xw-label">At a glance</h4>
          <dl class="ka-facts xw-facts">
            {#if !exchange}
              <div>
                <dt>Content</dt>
                <dd class="ka-between">
                  <span
                    >{countLabel(selected.length, "chapter")} · {countLabel(
                      sceneCount,
                      "scene"
                    )}</span
                  ><button
                    type="button"
                    class="ka-button ka-button--ghost"
                    aria-label="Change content"
                    onclick={() => (section = "content")}>Change</button
                  >
                </dd>
              </div>
              {#if !plain}
                <div>
                  <dt>Text &amp; page</dt>
                  <dd class="ka-between">
                    <span>{draft.font} · {draft.fontSize} pt</span><button
                      type="button"
                      class="ka-button ka-button--ghost"
                      aria-label="Change text and page"
                      onclick={() => (section = "text")}>Change</button
                    >
                  </dd>
                </div>
              {/if}
              <div>
                <dt>Headings &amp; breaks</dt>
                <dd class="ka-between">
                  <span>{draft.separator || "Blank line"}</span><button
                    type="button"
                    class="ka-button ka-button--ghost"
                    aria-label="Change headings and breaks"
                    onclick={() => (section = "headings")}>Change</button
                  >
                </dd>
              </div>
            {:else}
              <div>
                <dt>Content</dt>
                <dd class="ka-between">
                  <span>Whole project</span><button
                    type="button"
                    class="ka-button ka-button--ghost"
                    aria-label="Change content"
                    onclick={() => (section = "files")}>Change</button
                  >
                </dd>
              </div>
            {/if}
            <div>
              <dt>Output</dt>
              <dd class="ka-between">
                <span>{formatLabels[draft.format]}</span><button
                  type="button"
                  class="ka-button ka-button--ghost"
                  aria-label="Change output"
                  onclick={() => (section = "files")}>Change</button
                >
              </dd>
            </div>
          </dl>
          <p class="ka-help">
            {exchange
              ? "This format uses its dedicated exporter to preserve project structure and metadata. Configure the handoff in Files & format."
              : "Your selected settings are applied to the exported file. The on-screen preview is an approximation of Word pagination and ebook reading systems."}
          </p>
        </div>
      {:else if section === "content"}
        <div class="xw-group">
          <div class="ka-field od-field">
            <label for="export-scope">Include in this export</label><select
              id="export-scope"
              bind:value={draft.selection}
              onchange={() => {
                draft.sceneId = null;
                if (draft.selection === "selected" && !draft.chapterIds.length)
                  draft.chapterIds = chapters.map((c) => c.id);
              }}
              ><option value="all">Entire manuscript</option><option value="selected"
                >Selected chapters</option
              ></select
            >
          </div>
          {#if contextOverride}<div class="ka-notice od-row-top">
              <Info class="w-5 h-5" aria-hidden="true" />
              <div class="od-field od-fill">
                <p>
                  Opened for your selected {scope}. Saving this profile will remember that
                  selection.
                </p>
              </div>
            </div>{/if}
          {#if draft.sceneId}<div class="ka-notice od-row-top">
              <Info class="w-5 h-5" aria-hidden="true" />
              <div class="od-field od-fill xw-notice-body">
                <p>Only the selected scene is included.</p>
                <button
                  type="button"
                  class="ka-button ka-button--ghost"
                  onclick={() => (draft.sceneId = null)}>Include the rest of its chapter</button
                >
              </div>
            </div>{/if}
          <div class="chapter-list" role="group" aria-label="Chapter selection">
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
          <h4 class="xw-label">Alongside the prose</h4>
          <div class="xw-checks">
            <label class="ka-check xw-check"
              ><input
                id="export-scene-titles"
                type="checkbox"
                bind:checked={draft.sceneTitles}
              /><span class="xw-check-text"
                ><span>Scene titles</span><small>Use your scene names as headings.</small></span
              ></label
            >
            <label class="ka-check xw-check"
              ><input type="checkbox" bind:checked={draft.synopses} /><span class="xw-check-text"
                ><span>Scene synopses</span><small>Add the scene summary before its prose.</small
                ></span
              ></label
            >
            <label class="ka-check xw-check"
              ><input id="export-beats" type="checkbox" bind:checked={draft.beatHeadings} /><span
                class="xw-check-text"
                ><span>Beat headings</span><small>Page View outlines follow the scene prose.</small
                ></span
              ></label
            >
          </div>
          <p class="ka-help">
            Archived chapters and scenes, unused scenes, notes, to-dos, and editorial comments are
            excluded from the manuscript export.
          </p>
        </div>
      {:else if section === "headings"}
        <div class="xw-group">
          <div class="ka-field od-field">
            <label for="export-heading">Chapter heading</label><select
              id="export-heading"
              bind:value={draft.chapterHeading}
              ><option value="number_title">Number and title</option><option value="number"
                >Number only</option
              ><option value="title">Title only</option><option value="none">No heading</option
              ></select
            >
            <p class="ka-help">
              {draft.chapterHeading === "number_title"
                ? "Chapter 1: The Crossing"
                : draft.chapterHeading === "number"
                  ? "Chapter 1"
                  : draft.chapterHeading === "title"
                    ? "The Crossing"
                    : "Prose begins without a chapter heading"}
            </p>
          </div>
          {#if draft.chapterHeading.startsWith("number")}<div class="xw-pair">
              <div class="ka-field od-field">
                <label for="export-number-style">Number style</label><select
                  id="export-number-style"
                  bind:value={draft.numberStyle}
                  ><option value="arabic">1, 2, 3</option><option value="roman">I, II, III</option
                  ></select
                >
              </div>
              <div class="ka-field od-field">
                <label for="export-number-start">Start at</label><input
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
            <label for="export-separator">Between scenes</label><input
              id="export-separator"
              bind:value={draft.separator}
              maxlength="80"
              aria-describedby="export-separator-help"
            />
            <div class="xw-chips" role="group" aria-label="Scene break markers">
              {#each ["#", "* * *", "⁂", ""] as marker (marker)}<button
                  type="button"
                  class="ka-tag"
                  aria-pressed={draft.separator === marker}
                  onclick={() => (draft.separator = marker)}>{marker || "Blank line"}</button
                >{/each}
            </div>
            <p class="ka-help" id="export-separator-help">
              A centered marker separates scenes within a chapter.
            </p>
          </div>
          <div class="xw-checks">
            <label class="ka-check xw-check"
              ><input type="checkbox" bind:checked={draft.partTitles} /><span class="xw-check-text"
                ><span>Include Part titles</span><small
                  >Keep the larger movements of your story.</small
                ></span
              ></label
            >
            {#if draft.format === "docx"}<label class="ka-check xw-check"
                ><input
                  id="export-chapter-breaks"
                  type="checkbox"
                  bind:checked={draft.chapterBreaks}
                /><span class="xw-check-text"
                  ><span>Start chapters on a new page</span><small
                    >Dashed boundaries in preview; page breaks when printing HTML.</small
                  ></span
                ></label
              >{/if}
          </div>
        </div>
      {:else if section === "text"}
        <div class="xw-group">
          <div class="xw-pair">
            <div class="ka-field od-field">
              <label for="export-font">Body font</label><select
                id="export-font"
                bind:value={draft.font}
                >{#each ["Times New Roman", "Courier New", "Georgia", "Arial"] as font (font)}<option
                    >{font}</option
                  >{/each}</select
              >
            </div>
            <div class="ka-field od-field">
              <label for="export-font-size">Size (pt)</label><input
                id="export-font-size"
                type="number"
                min="8"
                max="24"
                bind:value={draft.fontSize}
              />
            </div>
          </div>
          <div class="ka-field od-field">
            <label for="export-spacing">Line spacing</label><select
              id="export-spacing"
              bind:value={draft.lineSpacing}
              ><option value={1}>Single</option><option value={1.5}>1.5 lines</option><option
                value={1.6}>Comfortable · 1.6 lines</option
              ><option value={2}>Double</option><option value={2.5}>2.5 lines</option></select
            >
          </div>
          <div class="xw-pair">
            <div class="ka-field od-field">
              <label for="export-indent">First-line indent (in)</label><input
                id="export-indent"
                type="number"
                min="0"
                max="1"
                step="0.05"
                bind:value={draft.indent}
              />
            </div>
            <div class="ka-field od-field">
              <label for="export-paragraph-spacing">After paragraphs (pt)</label><input
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
            ><input type="checkbox" bind:checked={draft.firstParagraphFlush} /> No indent after a heading
            or scene break</label
          >
          <details class="xw-details">
            <summary
              ><span>More text options</span><span class="xw-meta"
                >{draft.alignment === "left" ? "Left aligned" : "Justified"}</span
              ></summary
            >
            <div class="ka-field od-field">
              <label for="export-alignment">Body alignment</label><select
                id="export-alignment"
                bind:value={draft.alignment}
                ><option value="left">Left aligned</option><option value="justify">Justified</option
                ></select
              >
            </div>
          </details>
        </div>
        {#if draft.format === "docx"}<div class="xw-group">
            <h4 class="xw-label">Page layout</h4>
            <div class="xw-pair">
              <div class="ka-field od-field">
                <label for="export-paper">Paper size</label><select
                  id="export-paper"
                  bind:value={draft.paper}
                  ><option value="letter">US Letter</option><option value="a4">A4</option></select
                >
              </div>
              <div class="ka-field od-field">
                <label for="export-margin">Margins (in)</label><input
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
              Word output uses your paper size, margins, chapter breaks, and repeating header. This
              flowing preview does not reproduce Word pagination or page numbers.
            </p>
          </div>{:else}<p class="ka-help xw-note">
            Reflowable output has no fixed paper size. An ebook reader may override your font and
            spacing preferences.
          </p>{/if}
        {#if draft.format === "html" && !draft.styled}<div class="ka-notice od-row-top xw-note">
            <Info class="w-5 h-5" aria-hidden="true" />
            <div class="od-field od-fill xw-notice-body">
              <p>HTML styling is off. Enable styling to see these typography settings.</p>
              <button
                type="button"
                class="ka-button ka-button--ghost"
                onclick={() => (draft.styled = true)}>Enable styling</button
              >
            </div>
          </div>{/if}
      {:else if section === "details"}
        <div class="xw-group">
          <div class="ka-field od-field">
            <label for="export-title">Book title</label><input
              id="export-title"
              bind:value={draft.title}
            />
          </div>
          <div class="ka-field od-field">
            <label for="export-author">Author / pen name</label><input
              id="export-author"
              bind:value={draft.author}
              placeholder="Your publishing name"
            />
          </div>
          <label class="ka-check xw-check"
            ><input type="checkbox" bind:checked={draft.titlePage} /><span class="xw-check-text"
              ><span>{plain ? "Include a title block" : "Include a title page"}</span><small
                >Choose “Whole selection” in the preview to see it.</small
              ></span
            ></label
          >
          {#if draft.titlePage}<div class="ka-field od-field">
              <label for="export-subtitle"
                >Subtitle <span class="ka-optional">(optional)</span></label
              ><input id="export-subtitle" bind:value={draft.subtitle} />
            </div>
            <div class="ka-field od-field">
              <label for="export-word-count">Title-page word count</label><select
                id="export-word-count"
                bind:value={draft.wordCount}
                ><option value="rounded">Rounded to nearest 1,000</option><option value="exact"
                  >Exact prose count</option
                ><option value="none">Omit</option></select
              >
            </div>{/if}
          {#if draft.format === "docx"}<div class="ka-field od-field">
              <label for="export-header">Running header</label><select
                id="export-header"
                bind:value={draft.header}
                ><option value="author_title">Author / Title</option><option value="title"
                  >Title only</option
                ><option value="none">None</option></select
              >
              <p class="ka-help">
                Repeats in Word, with no header on the title page. Shown once in this preview.
              </p>
            </div>{/if}
          <label class="ka-check xw-check"
            ><input id="export-contents" type="checkbox" bind:checked={draft.contents} /><span
              class="xw-check-text"
              ><span>Table of contents</span><small
                >{plain
                  ? "Lists the chapter titles in your selection."
                  : "Links to the chapters in your full selection."}</small
              ></span
            ></label
          >
          {#if draft.format === "epub" || draft.format === "html"}
            <details class="xw-details">
              <summary><span>Language</span><span class="xw-meta">{draft.language}</span></summary>
              <div class="ka-field od-field">
                <label for="export-language">Document language</label><select
                  id="export-language"
                  bind:value={draft.language}
                  ><option value="en">English</option><option value="en-GB">English (UK)</option
                  ><option value="fr">French</option><option value="de">German</option><option
                    value="es">Spanish</option
                  ><option value="pt">Portuguese</option></select
                >
              </div>
            </details>
          {/if}
        </div>
        {#if draft.format === "epub"}<div class="xw-group">
            <h4 class="xw-label">Ebook metadata</h4>
            <div class="ka-field od-field">
              <label for="export-description">Description</label><textarea
                id="export-description"
                rows="3"
                bind:value={draft.description}
              ></textarea>
            </div>
            <div class="ka-field od-field">
              <label for="export-cover">Cover image</label>
              <div class="xw-location">
                <input
                  id="export-cover"
                  readonly
                  value={draft.coverPath}
                  placeholder="Optional PNG or JPEG"
                  aria-describedby="export-cover-help"
                />
                <button type="button" class="ka-button ka-button--secondary" onclick={chooseCover}
                  >Choose cover</button
                >{#if draft.coverPath}<button
                    type="button"
                    class="ka-button ka-button--ghost"
                    onclick={() => (draft.coverPath = "")}>Remove</button
                  >{/if}
              </div>
              <p class="ka-help" id="export-cover-help">
                The cover is embedded in the EPUB; it is not shown in the manuscript preview.
              </p>
            </div>
          </div>
        {/if}
      {:else if section === "files"}
        <div class="xw-group">
          <div class="ka-field od-field">
            <label for="export-filename">Filename pattern</label><input
              id="export-filename"
              bind:value={draft.fileName}
              aria-describedby="export-filename-sample"
            />
            <div class="xw-chips" role="group" aria-label="Filename tokens">
              {#each ["{title}", "{profile}", "{date}"] as token (token)}<button
                  type="button"
                  class="ka-tag"
                  onclick={() => (draft.fileName += token)}>+ {token}</button
                >{/each}
            </div>
            <p class="ka-help filename-sample" id="export-filename-sample">{fileName}</p>
          </div>
          {#if draft.format === "html"}<div class="ka-field od-field">
              <label for="export-html-mode">HTML structure</label><select
                id="export-html-mode"
                bind:value={draft.htmlMode}
                ><option value="document">Complete HTML document</option><option value="fragment"
                  >Body fragment for pasting</option
                ></select
              >
              <p class="ka-help">Both are saved as a single .html file</p>
            </div>
            <div class="ka-field od-field">
              <label for="export-html-heading">Chapter heading element</label><select
                id="export-html-heading"
                bind:value={draft.headingLevel}
                ><option value="h1">Heading 1 · &lt;h1&gt;</option><option value="h2"
                  >Heading 2 · &lt;h2&gt;</option
                ></select
              >
            </div>
            <label class="ka-check xw-check"
              ><input type="checkbox" bind:checked={draft.styled} /><span class="xw-check-text"
                ><span>Include built-in styling</span><small
                  >Turn off for clean semantic markup.</small
                ></span
              ></label
            >{/if}
          {#if draft.format === "novelwriter"}
            <div class="xw-checks">
              <label class="ka-check xw-check"
                ><input type="checkbox" bind:checked={draft.includeNotes} /><span
                  class="xw-check-text"
                  ><span>Reference notes</span><small
                    >Include character, location, and reference material.</small
                  ></span
                ></label
              >
              <label class="ka-check xw-check"
                ><input type="checkbox" bind:checked={draft.includeBeatComments} /><span
                  class="xw-check-text"
                  ><span>Beat comments</span><small>Retain outline prompts as comments.</small
                  ></span
                ></label
              >
            </div>
          {:else if draft.format === "treatment"}
            <div class="ka-field od-field">
              <label for="export-treatment-level">Treatment detail</label><select
                id="export-treatment-level"
                bind:value={draft.treatmentLevel}
                ><option value="one_page">One page · overview</option><option value="five_page"
                  >Five pages · key scenes</option
                ><option value="full">Full · scenes and beats</option></select
              >
              <p class="ka-help">
                Detail levels guide the content; page counts depend on the material.
              </p>
            </div>
            <div class="ka-field od-field">
              <label for="export-treatment-format">Treatment file</label><select
                id="export-treatment-format"
                bind:value={draft.treatmentFormat}
                ><option value="docx">Word document (.docx)</option><option value="txt"
                  >Plain text (.txt)</option
                ></select
              >
            </div>
          {:else if draft.format === "scrivener"}<p class="ka-help">
              Creates a new .scriv project. To update an existing Scrivener project with scene
              matching and backups, use Back to export.
            </p>
          {:else if draft.format === "longform"}<p class="ka-help">
              Creates a Longform index, individual scene files, and reference notes. Project and
              round-trip metadata are retained.
            </p>{/if}
        </div>
        <div class="xw-group">
          <div class="xw-file">
            <FileText class="w-5 h-5" aria-hidden="true" />
            <div class="od-field od-fill">
              <strong>{formatLabels[draft.format]}</strong><span class="xw-meta"
                >{exchange
                  ? "Whole project · dedicated exporter"
                  : `${countLabel(selected.length, "chapter")} · ${countLabel(sceneCount, "scene")}`}</span
              >
            </div>
            <span class="xw-file-type"
              >{["longform", "novelwriter"].includes(draft.format)
                ? "Folder"
                : `.${outputExtension(draft)}`}</span
            >
          </div>
          <p class="ka-help">
            Choose a destination when exporting. Existing files and folders are preserved; use a new
            name for each export.
          </p>
          {#if draft.format === "markdown"}<p class="ka-help">
              Exports a single manuscript file with headings, emphasis, lists, and block quotations.
              For an Obsidian project with separate scene files and reference notes, choose Longform
              / Obsidian.
            </p>{/if}
        </div>
      {/if}
    </section>

    <section
      class="xw-preview"
      class:mobile-hidden={narrowPane === "settings"}
      aria-label="Manuscript preview"
    >
      {#if exchange}<div class="ka-empty od-stack empty-preview">
          <FileDown class="w-7 h-7" aria-hidden="true" />
          <h4>{formatLabels[draft.format]}</h4>
          <p>
            {draft.format === "treatment"
              ? "Builds a treatment from the whole project’s outline, synopses, and beats. Manuscript prose styling does not apply."
              : "Transfers the whole project using its existing structure and metadata. Manuscript formatting controls do not apply."}
          </p>
          <p class="filename-sample">{fileName}</p>
          <p>
            {draft.format === "scrivener"
              ? "A new Scrivener bundle with chapters, scenes, and RTF prose."
              : draft.format === "longform"
                ? "A Longform index, scene Markdown files, and reference folders."
                : draft.format === "novelwriter"
                  ? "nwProject.nwx, content documents, and selected notes."
                  : "Uses project title and author settings with the selected treatment detail."}
          </p>
        </div>{:else}
        <header class="xw-preview-head">
          <div class="xw-preview-title">
            <span class="xw-label">Live preview</span>
            <span class="ka-badge ka-badge--success">Live</span>
            {#if draft.format === "html"}<div
                class="ka-segment-track preview-tabs"
                role="group"
                aria-label="Preview view"
              >
                <button
                  type="button"
                  class="ka-segment"
                  class:ka-selected={previewMode === "rendered"}
                  aria-pressed={previewMode === "rendered"}
                  onclick={() => (previewMode = "rendered")}>Read</button
                ><button
                  type="button"
                  class="ka-segment"
                  class:ka-selected={previewMode === "source"}
                  aria-pressed={previewMode === "source"}
                  onclick={() => (previewMode = "source")}>HTML</button
                >
              </div>
            {/if}
          </div>
          <div class="xw-preview-controls">
            {#if plain}<p class="xw-meta od-fill">Whole selection · single file</p>{:else}<div
                class="ka-field od-field od-fill"
              >
                <label class="ka-sr" for="export-preview-chapter">Preview chapter</label><select
                  id="export-preview-chapter"
                  value={actualPreviewChapter}
                  onchange={(event) => (previewChapter = event.currentTarget.value)}
                  disabled={!selected.length}
                  ><option value=""
                    >{selected.length ? "Whole selection" : "No chapters selected"}</option
                  >{#each selected as chapter (chapter.id)}<option value={chapter.id}
                      >{chapter.title}</option
                    >{/each}</select
                >
              </div>{/if}<button
              type="button"
              class="ka-button ka-button--ghost ka-icon-button"
              aria-label="Refresh saved manuscript"
              title="Refresh saved manuscript"
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
              <h4>Gathering your manuscript…</h4>
            </div>{:else if !selected.length}<div class="ka-empty od-stack empty-preview">
              <BookOpen class="w-7 h-7" aria-hidden="true" />
              <h4>Nothing selected yet</h4>
              <p>Choose a chapter with manuscript scenes to see it here.</p>
              <button
                type="button"
                class="ka-button ka-button--secondary"
                onclick={() => {
                  section = "content";
                  narrowPane = "settings";
                }}>Choose content <ArrowRight class="w-5 h-5" aria-hidden="true" /></button
              >
            </div>{:else if (draft.format === "html" && previewMode === "source") || plain}<textarea
              class="source-preview"
              readonly
              aria-label={plain ? "Generated manuscript text" : "Generated HTML source"}
              value={sourceText}
            ></textarea>{:else}<iframe
              title="Export layout preview"
              sandbox=""
              srcdoc={previewDocument}
              data-ready={previewLoaded === previewDocument}
              onload={(event) => (previewLoaded = event.currentTarget.getAttribute("srcdoc") ?? "")}
            ></iframe>{/if}
        </div>
        <footer class="xw-preview-foot">
          <span
            >{draft.format === "html"
              ? "HTML output"
              : plain
                ? "Text output"
                : "Layout approximation"}</span
          ><span>{wordCount.toLocaleString()} words · {countLabel(sceneCount, "scene")}</span>
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
            {#if !loading && error.includes("load")}<button
                type="button"
                class="ka-button ka-button--ghost"
                onclick={() => loadManuscript()}>Retry</button
              >{/if}
          </div>
        </div>{/if}
      {#if missingSelection}<div class="ka-notice ka-notice--error od-row-top" role="alert">
          <TriangleAlert class="w-5 h-5" aria-hidden="true" />
          <div class="od-field od-fill">
            <p>Part of this saved selection is missing. Review Content before saving a preview.</p>
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
                >Open export <ArrowRight class="w-5 h-5" aria-hidden="true" /></button
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
      <div>{exchange ? "Whole project" : "Saved manuscript"} → {formatLabels[draft.format]}</div>
      <div>Profiles and recovered drafts are local to this device.</div>
    </div>
    <div class="ka-row xw-foot-actions">
      {#if dirty}<button
          type="button"
          class="ka-button ka-button--ghost"
          disabled={saving}
          onclick={() => {
            if (savedProfile) draft = window.structuredClone($state.snapshot(savedProfile));
            contextSelection = null;
          }}>Revert</button
        >{/if}<button
        type="button"
        class="ka-button ka-button--secondary"
        disabled={saving || storageBlocked || !!validationError}
        onclick={() => saveProfile()}>Save profile</button
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
        >{#if saving}<Loader2
            class="w-5 h-5 animate-spin"
            aria-hidden="true"
          />Exporting…{:else}<Download
            class="w-5 h-5"
            aria-hidden="true"
          />{`Export ${draft.format === "treatment" ? draft.treatmentFormat.toUpperCase() : draft.format === "longform" || draft.format === "novelwriter" ? "project" : outputExtension(draft).toUpperCase()}`}{/if}</button
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
