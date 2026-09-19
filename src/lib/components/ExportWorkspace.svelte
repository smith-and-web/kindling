<script lang="ts">
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
    Code,
    FilePlus2,
    FileDown,
    FileText,
    LayoutTemplate,
    ListTree,
    RefreshCw,
    Search,
    Settings2,
    Type,
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
    {
      id: "overview",
      label: "Overview",
      icon: LayoutTemplate,
      description: "A familiar starting point",
    },
    { id: "content", label: "Content", icon: ListTree, description: "Choose what goes in" },
    {
      id: "headings",
      label: "Headings & breaks",
      icon: BookOpen,
      description: "Give the story structure",
    },
    { id: "text", label: "Text & page", icon: Type, description: "Set the reading rhythm" },
    {
      id: "details",
      label: "Book details",
      icon: FileText,
      description: "Title, author & contents",
    },
    { id: "files", label: "Files & format", icon: FileDown, description: "Prepare the handoff" },
  ];
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
    const path = await open({
      title: "Choose an ebook cover",
      filters: [{ name: "Cover image", extensions: ["png", "jpg", "jpeg"] }],
    });
    if (path) draft.coverPath = path;
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
  <header class="workspace-header">
    <div class="identity">
      <span class="export-icon"><FileDown size={21} /></span>
      <div>
        <h1 id="export-workspace-title">Export workspace</h1>
      </div>
    </div>
    <div class="header-actions">
      <button class="quiet" onclick={onClassic} disabled={saving}
        ><ArrowLeft size={15} /> Back to export</button
      ><button
        class="icon-button"
        aria-label="Close export workspace"
        onclick={onClose}
        disabled={saving}><X size={21} /></button
      >
    </div>
  </header>

  <div class="profile-bar">
    <div class="profile-picker">
      <label for="export-profile">Export profile</label>
      <div class="inline">
        <select
          id="export-profile"
          value={draft.id}
          onchange={(event) => chooseProfile(event.currentTarget.value)}
          >{#each profiles as profile (profile.id)}<option value={profile.id}>{profile.name}</option
            >{/each}</select
        ><button
          class="icon-button duplicate-profile"
          aria-label="Duplicate profile"
          title="Duplicate profile"
          onclick={duplicateProfile}
          disabled={storageBlocked}><FilePlus2 size={16} /></button
        >
      </div>
    </div>
    <div class="format-picker">
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
    <div class="profile-status">
      <span class:unsaved={dirty}
        >{#if dirty}<span class="status-dot"></span> Unsaved changes{:else}<Check size={14} /> Profile
          saved{/if}</span
      ><small>Profiles stay on this device</small>
    </div>
  </div>

  <div class="mobile-switch" aria-label="Workspace view">
    <button class:active={narrowPane === "settings"} onclick={() => (narrowPane = "settings")}
      ><Settings2 size={15} /> Settings</button
    ><button class:active={narrowPane === "preview"} onclick={() => (narrowPane = "preview")}
      ><BookOpen size={15} /> Preview</button
    >
  </div>

  <div class="workspace-body">
    <aside class="navigation" class:mobile-hidden={narrowPane === "preview"}>
      <div class="search-box">
        <Search size={15} /><input
          aria-label="Find an export setting"
          placeholder="Find a setting…"
          bind:value={query}
        />{#if query}<button
            aria-label="Clear settings search"
            class="icon-button"
            onclick={() => (query = "")}><X size={14} /></button
          >{/if}
      </div>
      {#if query.trim()}
        <div class="search-results">
          <small>{searchResults.length} settings found</small>{#each searchResults as item}<button
              onclick={() => findSetting(item)}>{item.label}<ChevronRight size={14} /></button
            >{/each}{#if !searchResults.length}<p>Try “spacing,” “header,” or “HTML.”</p>{/if}
        </div>
      {:else}
        <nav aria-label="Export settings">
          {#each categories as category}<button
              class:active={section === category.id}
              aria-current={section === category.id ? "page" : undefined}
              onclick={() => (section = category.id)}
              ><category.icon size={17} /><span
                >{category.label}<small>{category.description}</small></span
              >{#if section === category.id}<ChevronRight size={14} />{/if}</button
            >{/each}
        </nav>
        <div class="sidebar-note">
          <BookOpen size={19} />
          <p>Make it yours.<br /><span>Your manuscript stays exactly as you wrote it.</span></p>
        </div>
      {/if}
    </aside>

    <section
      class="settings-panel"
      class:mobile-hidden={narrowPane === "preview"}
      aria-label="Export configuration"
    >
      <div class="section-heading">
        <div class="eyebrow">{formatLabels[draft.format]}</div>
        <h2>{visibleCategory.label}</h2>
        <p>{visibleCategory.description}.</p>
      </div>

      {#if section === "overview"}
        <div class="field">
          <label for="export-name">Profile name</label><input
            id="export-name"
            bind:value={draft.name}
            maxlength="100"
          /><small>Name it for a recipient or a writing routine.</small>
        </div>
        <h3 class="subheading">A starting point for every handoff</h3>
        <div class="preset-list">
          {#each profiles.slice(0, 3) as preset}<button
              class:selected={draft.id === preset.id}
              onclick={() => chooseProfile(preset.id)}
              ><span class="preset-icon"
                >{#if preset.format === "docx"}<FileText
                    size={21}
                  />{:else if preset.format === "epub"}<BookOpen size={21} />{:else}<Code
                    size={21}
                  />{/if}</span
              ><span
                ><strong>{preset.name}</strong><small
                  >{preset.format === "docx"
                    ? "A clean, familiar submission format"
                    : preset.format === "epub"
                      ? "A comfortable copy for your readers"
                      : "Clean markup for your next destination"}</small
                ></span
              >{#if draft.id === preset.id}<Check size={17} />{:else}<ArrowRight
                  size={16}
                />{/if}</button
            >{/each}
        </div>
        <h3 class="subheading">At a glance</h3>
        <div class="summary-list">
          {#if !exchange}<button onclick={() => (section = "content")}
              ><span>Content</span><strong>{selected.length} chapters · {sceneCount} scenes</strong
              ><ChevronRight size={14} /></button
            >
            {#if !plain}<button onclick={() => (section = "text")}
                ><span>Typography</span><strong>{draft.font} · {draft.fontSize} pt</strong
                ><ChevronRight size={14} /></button
              >{/if}
            <button onclick={() => (section = "headings")}
              ><span>Scene breaks</span><strong>{draft.separator || "Blank line"}</strong
              ><ChevronRight size={14} /></button
            >{:else}<button onclick={() => (section = "files")}
              ><span>Content</span><strong>Whole project</strong><ChevronRight size={14} /></button
            >{/if}
          <button onclick={() => (section = "files")}
            ><span>Output</span><strong>{formatLabels[draft.format]}</strong><ChevronRight
              size={14}
            /></button
          >
        </div>
        <p class="callout">
          {exchange
            ? "This format uses its dedicated exporter to preserve project structure and metadata. Configure the handoff in Files & format."
            : "Your selected settings are applied to the exported file. The on-screen preview is an approximation of Word pagination and ebook reading systems."}
        </p>
      {:else if section === "content"}
        <div class="field">
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
        {#if contextOverride}<p class="callout">
            Opened for your selected {scope}. Saving this profile will remember that selection.
          </p>{/if}
        {#if draft.sceneId}<p class="callout">
            Only the selected scene is included. <button
              class="text-button"
              onclick={() => (draft.sceneId = null)}>Include the rest of its chapter</button
            >
          </p>{/if}
        <div class="chapter-list" aria-label="Chapter selection">
          {#each chapters as chapter}<label class="chapter-row"
              ><input
                type="checkbox"
                checked={draft.selection === "all" || draft.chapterIds.includes(chapter.id)}
                disabled={draft.selection === "all"}
                onchange={(event) => selectChapter(chapter.id, event.currentTarget.checked)}
              /><span
                >{chapter.title}<small
                  >{chapter.scenes.length} scenes{chapter.part ? ` · ${chapter.part}` : ""}</small
                ></span
              ></label
            >{/each}
        </div>
        <h3 class="subheading">Alongside the prose</h3>
        <label class="check-row"
          ><input id="export-scene-titles" type="checkbox" bind:checked={draft.sceneTitles} /><span
            >Scene titles<small>Use your scene names as headings.</small></span
          ></label
        >
        <label class="check-row"
          ><input type="checkbox" bind:checked={draft.synopses} /><span
            >Scene synopses<small>Add the scene summary before its prose.</small></span
          ></label
        >
        <label class="check-row"
          ><input id="export-beats" type="checkbox" bind:checked={draft.beatHeadings} /><span
            >Beat headings<small>Page View outlines follow the scene prose.</small></span
          ></label
        >
        <p class="footnote">
          Archived chapters and scenes, unused scenes, notes, to-dos, and editorial comments are
          excluded from the manuscript export.
        </p>
      {:else if section === "headings"}
        <div class="field">
          <label for="export-heading">Chapter heading</label><select
            id="export-heading"
            bind:value={draft.chapterHeading}
            ><option value="number_title">Number and title</option><option value="number"
              >Number only</option
            ><option value="title">Title only</option><option value="none">No heading</option
            ></select
          ><small
            >{draft.chapterHeading === "number_title"
              ? "Chapter 1: The Crossing"
              : draft.chapterHeading === "number"
                ? "Chapter 1"
                : draft.chapterHeading === "title"
                  ? "The Crossing"
                  : "Prose begins without a chapter heading"}</small
          >
        </div>
        {#if draft.chapterHeading.startsWith("number")}<div class="field-grid">
            <div class="field">
              <label for="export-number-style">Number style</label><select
                id="export-number-style"
                bind:value={draft.numberStyle}
                ><option value="arabic">1, 2, 3</option><option value="roman">I, II, III</option
                ></select
              >
            </div>
            <div class="field">
              <label for="export-number-start">Start at</label><input
                id="export-number-start"
                type="number"
                min="1"
                max="999"
                bind:value={draft.startNumber}
              />
            </div>
          </div>{/if}
        <div class="field">
          <label for="export-separator">Between scenes</label><input
            id="export-separator"
            bind:value={draft.separator}
            maxlength="80"
          />
          <div class="chip-row">
            {#each ["#", "* * *", "⁂", ""] as marker}<button
                class:chosen={draft.separator === marker}
                onclick={() => (draft.separator = marker)}>{marker || "Blank line"}</button
              >{/each}
          </div>
          <small>A centered marker separates scenes within a chapter.</small>
        </div>
        <label class="check-row"
          ><input type="checkbox" bind:checked={draft.partTitles} /><span
            >Include Part titles<small>Keep the larger movements of your story.</small></span
          ></label
        >
        {#if draft.format === "docx"}<label class="check-row"
            ><input
              id="export-chapter-breaks"
              type="checkbox"
              bind:checked={draft.chapterBreaks}
            /><span
              >Start chapters on a new page<small
                >Dashed boundaries in preview; page breaks when printing HTML.</small
              ></span
            ></label
          >{/if}
      {:else if section === "text"}
        <div class="field-grid">
          <div class="field">
            <label for="export-font">Body font</label><select
              id="export-font"
              bind:value={draft.font}
              >{#each ["Times New Roman", "Courier New", "Georgia", "Arial"] as font}<option
                  >{font}</option
                >{/each}</select
            >
          </div>
          <div class="field">
            <label for="export-font-size">Size (pt)</label><input
              id="export-font-size"
              type="number"
              min="8"
              max="24"
              bind:value={draft.fontSize}
            />
          </div>
        </div>
        <div class="field">
          <label for="export-spacing">Line spacing</label><select
            id="export-spacing"
            bind:value={draft.lineSpacing}
            ><option value={1}>Single</option><option value={1.5}>1.5 lines</option><option
              value={1.6}>Comfortable · 1.6 lines</option
            ><option value={2}>Double</option><option value={2.5}>2.5 lines</option></select
          >
        </div>
        <div class="field-grid">
          <div class="field">
            <label for="export-indent">First-line indent (in)</label><input
              id="export-indent"
              type="number"
              min="0"
              max="1"
              step="0.05"
              bind:value={draft.indent}
            />
          </div>
          <div class="field">
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
        <label class="check-row"
          ><input type="checkbox" bind:checked={draft.firstParagraphFlush} /><span
            >No indent after a heading or scene break</span
          ></label
        >
        <details>
          <summary
            >More text options <span
              >{draft.alignment === "left" ? "Left aligned" : "Justified"}</span
            ></summary
          >
          <div class="field">
            <label for="export-alignment">Body alignment</label><select
              id="export-alignment"
              bind:value={draft.alignment}
              ><option value="left">Left aligned</option><option value="justify">Justified</option
              ></select
            >
          </div>
        </details>
        {#if draft.format === "docx"}<h3 class="subheading">Page layout</h3>
          <div class="field-grid">
            <div class="field">
              <label for="export-paper">Paper size</label><select
                id="export-paper"
                bind:value={draft.paper}
                ><option value="letter">US Letter</option><option value="a4">A4</option></select
              >
            </div>
            <div class="field">
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
          <p class="footnote">
            Word output uses your paper size, margins, chapter breaks, and repeating header. This
            flowing preview does not reproduce Word pagination or page numbers.
          </p>{:else}<p class="footnote">
            Reflowable output has no fixed paper size. An ebook reader may override your font and
            spacing preferences.
          </p>{/if}
        {#if draft.format === "html" && !draft.styled}<p class="callout">
            HTML styling is off. <button class="text-button" onclick={() => (draft.styled = true)}
              >Enable styling</button
            > to see these typography settings.
          </p>{/if}
      {:else if section === "details"}
        <div class="field">
          <label for="export-title">Book title</label><input
            id="export-title"
            bind:value={draft.title}
          />
        </div>
        <div class="field">
          <label for="export-author">Author / pen name</label><input
            id="export-author"
            bind:value={draft.author}
            placeholder="Your publishing name"
          />
        </div>
        <label class="check-row"
          ><input type="checkbox" bind:checked={draft.titlePage} /><span
            >{plain ? "Include a title block" : "Include a title page"}<small
              >Choose “Whole selection” in the preview to see it.</small
            ></span
          ></label
        >
        {#if draft.titlePage}<div class="field">
            <label for="export-subtitle">Subtitle (optional)</label><input
              id="export-subtitle"
              bind:value={draft.subtitle}
            />
          </div>
          <div class="field">
            <label for="export-word-count">Title-page word count</label><select
              id="export-word-count"
              bind:value={draft.wordCount}
              ><option value="rounded">Rounded to nearest 1,000</option><option value="exact"
                >Exact prose count</option
              ><option value="none">Omit</option></select
            >
          </div>{/if}
        {#if draft.format === "docx"}<div class="field">
            <label for="export-header">Running header</label><select
              id="export-header"
              bind:value={draft.header}
              ><option value="author_title">Author / Title</option><option value="title"
                >Title only</option
              ><option value="none">None</option></select
            ><small
              >Repeats in Word, with no header on the title page. Shown once in this preview.</small
            >
          </div>{/if}
        <label class="check-row"
          ><input id="export-contents" type="checkbox" bind:checked={draft.contents} /><span
            >Table of contents<small
              >{plain
                ? "Lists the chapter titles in your selection."
                : "Links to the chapters in your full selection."}</small
            ></span
          ></label
        >
        {#if draft.format === "epub" || draft.format === "html"}
          <details>
            <summary>Language <span>{draft.language}</span></summary>
            <div class="field">
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
        {#if draft.format === "epub"}<h3 class="subheading">Ebook metadata</h3>
          <div class="field">
            <label for="export-description">Description</label><textarea
              id="export-description"
              rows="3"
              bind:value={draft.description}
            ></textarea>
          </div>
          <div class="field">
            <label for="export-cover">Cover image</label><input
              id="export-cover"
              readonly
              value={draft.coverPath}
              placeholder="Optional PNG or JPEG"
            />
            <div class="inline">
              <button class="secondary" onclick={chooseCover}>Choose cover</button
              >{#if draft.coverPath}<button class="quiet" onclick={() => (draft.coverPath = "")}
                  >Remove</button
                >{/if}
            </div>
            <small
              >The cover is embedded in the EPUB; it is not shown in the manuscript preview.</small
            >
          </div>
        {/if}
      {:else if section === "files"}
        <div class="field">
          <label for="export-filename">Filename pattern</label><input
            id="export-filename"
            bind:value={draft.fileName}
          />
          <div class="chip-row">
            {#each ["{title}", "{profile}", "{date}"] as token}<button
                onclick={() => (draft.fileName += token)}>+ {token}</button
              >{/each}
          </div>
          <small class="filename-sample">{fileName}</small>
        </div>
        {#if draft.format === "html"}<div class="field">
            <label for="export-html-mode">HTML structure</label><select
              id="export-html-mode"
              bind:value={draft.htmlMode}
              ><option value="document">Complete HTML document</option><option value="fragment"
                >Body fragment for pasting</option
              ></select
            ><small>Both are saved as a single .html file </small>
          </div>
          <div class="field">
            <label for="export-html-heading">Chapter heading element</label><select
              id="export-html-heading"
              bind:value={draft.headingLevel}
              ><option value="h1">Heading 1 · &lt;h1&gt;</option><option value="h2"
                >Heading 2 · &lt;h2&gt;</option
              ></select
            >
          </div>
          <label class="check-row"
            ><input type="checkbox" bind:checked={draft.styled} /><span
              >Include built-in styling<small>Turn off for clean semantic markup.</small></span
            ></label
          >{/if}
        {#if draft.format === "novelwriter"}
          <label class="check-row"
            ><input type="checkbox" bind:checked={draft.includeNotes} /><span
              >Reference notes<small>Include character, location, and reference material.</small
              ></span
            ></label
          >
          <label class="check-row"
            ><input type="checkbox" bind:checked={draft.includeBeatComments} /><span
              >Beat comments<small>Retain outline prompts as comments.</small></span
            ></label
          >
        {:else if draft.format === "treatment"}
          <div class="field">
            <label for="export-treatment-level">Treatment detail</label><select
              id="export-treatment-level"
              bind:value={draft.treatmentLevel}
              ><option value="one_page">One page · overview</option><option value="five_page"
                >Five pages · key scenes</option
              ><option value="full">Full · scenes and beats</option></select
            ><small>Detail levels guide the content; page counts depend on the material.</small>
          </div>
          <div class="field">
            <label for="export-treatment-format">Treatment file</label><select
              id="export-treatment-format"
              bind:value={draft.treatmentFormat}
              ><option value="docx">Word document (.docx)</option><option value="txt"
                >Plain text (.txt)</option
              ></select
            >
          </div>
        {:else if draft.format === "scrivener"}<p class="callout">
            Creates a new .scriv project. To update an existing Scrivener project with scene
            matching and backups, use Back to export.
          </p>
        {:else if draft.format === "longform"}<p class="callout">
            Creates a Longform index, individual scene files, and reference notes. Project and
            round-trip metadata are retained.
          </p>{/if}
        <div class="file-card">
          <FileText size={24} />
          <div>
            <strong>{formatLabels[draft.format]}</strong><small
              >{exchange
                ? "Whole project · dedicated exporter"
                : `${selected.length} chapters · ${sceneCount} scenes`}</small
            >
          </div>
          <span
            >{["longform", "novelwriter"].includes(draft.format)
              ? "Folder"
              : `.${outputExtension(draft)}`}</span
          >
        </div>
        <p class="callout">
          Choose a destination when exporting. Existing files and folders are preserved; use a new
          name for each export.
        </p>
        {#if draft.format === "markdown"}<p class="footnote">
            Exports a single manuscript file with headings, emphasis, lists, and block quotations.
            For an Obsidian project with separate scene files and reference notes, choose Longform /
            Obsidian.
          </p>{/if}
      {/if}
    </section>

    <section
      class="preview-panel"
      class:mobile-hidden={narrowPane === "settings"}
      aria-label="Manuscript preview"
    >
      {#if exchange}<div class="empty-preview">
          <FileDown size={32} />
          <h3>{formatLabels[draft.format]}</h3>
          <p>
            {draft.format === "treatment"
              ? "Builds a treatment from the whole project’s outline, synopses, and beats. Manuscript prose styling does not apply."
              : "Transfers the whole project using its existing structure and metadata. Manuscript formatting controls do not apply."}
          </p>
          <div class="file-card">
            <FileText size={22} /><strong class="filename-sample">{fileName}</strong>
          </div>
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
        <div class="preview-toolbar">
          <div><span class="eyebrow">LIVE PREVIEW</span><span class="live-dot"></span></div>
          {#if draft.format === "html"}<div class="preview-tabs">
              <button
                class:active={previewMode === "rendered"}
                aria-pressed={previewMode === "rendered"}
                onclick={() => (previewMode = "rendered")}><BookOpen size={14} /> Read</button
              ><button
                class:active={previewMode === "source"}
                aria-pressed={previewMode === "source"}
                onclick={() => (previewMode = "source")}><Code size={14} /> HTML</button
              >
            </div>
          {/if}
        </div>
        <div class="preview-selection">
          {#if plain}<span>Whole selection · single file</span>{:else}<label
              class="sr-only"
              for="export-preview-chapter">Preview chapter</label
            ><select id="export-preview-chapter" bind:value={previewChapter}
              ><option value="">Whole selection</option>{#each selected as chapter}<option
                  value={chapter.id}>{chapter.title}</option
                >{/each}</select
            >{/if}<button
            class="icon-button"
            aria-label="Refresh saved manuscript"
            title="Refresh saved manuscript"
            disabled={loading}
            onclick={() => loadManuscript()}
            ><RefreshCw size={15} class={loading ? "spinning" : ""} /></button
          >
        </div>
        {#if loading}<div class="empty-preview">
            <RefreshCw class="spinning" size={26} />
            <h3>Gathering your manuscript…</h3>
          </div>{:else if !selected.length}<div class="empty-preview">
            <BookOpen size={32} />
            <h3>Nothing selected yet</h3>
            <p>Choose a chapter with manuscript scenes to see it here.</p>
            <button
              onclick={() => {
                section = "content";
                narrowPane = "settings";
              }}>Choose content <ArrowRight size={15} /></button
            >
          </div>{:else if (draft.format === "html" && previewMode === "source") || plain}<textarea
            class="source-preview"
            readonly
            aria-label={plain ? "Generated manuscript text" : "Generated HTML source"}
            value={sourceText}
          ></textarea>{:else}<iframe
            title="Export layout preview"
            sandbox=""
            srcdoc={preview.document}
          ></iframe>{/if}
        <div class="preview-caption">
          <span
            >{draft.format === "html"
              ? "HTML output"
              : plain
                ? "Text output"
                : "Layout approximation"}</span
          ><span>{wordCount.toLocaleString()} words · {sceneCount} scenes</span>
        </div>
      {/if}
    </section>
  </div>

  {#if error || storageError}<div class="feedback error" role="alert">
      {error || storageError}{#if !loading && error.includes("load")}<button
          class="text-button"
          onclick={() => loadManuscript()}>Retry</button
        >{/if}
    </div>{/if}
  {#if missingSelection}<div class="feedback error" role="alert">
      Part of this saved selection is missing. Review Content before saving a preview.
    </div>{/if}
  {#if message}<div class="feedback" role="status">
      {message}{#if savedPath}<button class="text-button" onclick={openSaved}
          >Open export <ArrowRight size={14} /></button
        >{/if}
    </div>{/if}
  {#if unsupported}<div class="feedback error" role="alert">{unsupported}</div>{/if}
  {#if validationError}<div class="feedback error" role="alert">{validationError}</div>{/if}
  <footer class="workspace-footer">
    <div class="footer-note">
      <span
        >{exchange ? "Whole project" : "Saved manuscript"} → {formatLabels[draft.format]}<br
        /><small>Profiles and recovered drafts are local to this device.</small></span
      >
    </div>
    <div class="footer-actions">
      {#if dirty}<button
          class="quiet"
          disabled={saving}
          onclick={() => {
            if (savedProfile) draft = window.structuredClone($state.snapshot(savedProfile));
            contextSelection = null;
          }}>Revert</button
        >{/if}<button
        class="secondary"
        disabled={saving || storageBlocked || !!validationError}
        onclick={() => saveProfile()}>Save profile</button
      ><button
        class="primary"
        disabled={saving ||
          loading ||
          !documentLoaded ||
          (!exchange && !sceneCount) ||
          !!unsupported ||
          missingSelection ||
          !!validationError}
        onclick={saveExport}
        ><FileDown size={16} />{saving
          ? "Exporting…"
          : `Export ${draft.format === "treatment" ? draft.treatmentFormat.toUpperCase() : draft.format === "longform" || draft.format === "novelwriter" ? "project" : outputExtension(draft).toUpperCase()}`}</button
      >
    </div>
  </footer>
</dialog>

<style>
  .export-workspace {
    width: calc(100vw - 40px);
    max-width: 1510px;
    height: calc(100vh - 40px);
    max-height: 1040px;
    padding: 0;
    margin: auto;
    border: 1px solid var(--color-border);
    border-radius: 12px;
    background: var(--color-surface);
    color: var(--color-text);
    overflow: hidden;
    font-family: var(--font-ui);
    font-size: var(--text-ui);
  }
  .export-workspace[open] {
    display: flex;
    flex-direction: column;
  }
  .export-workspace::backdrop {
    background: var(--color-overlay-scrim);
    backdrop-filter: blur(4px);
  }
  button,
  input,
  select {
    font: inherit;
  }
  button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    cursor: pointer;
    border-radius: 6px;
    transition: background 120ms;
  }
  button:disabled {
    cursor: default;
    background: var(--color-disabled-bg);
    color: var(--color-disabled-text);
    border-color: var(--color-disabled-border);
  }
  button:focus-visible,
  input:focus-visible,
  select:focus-visible,
  summary:focus-visible,
  textarea:focus-visible {
    outline: 2px solid var(--color-control-border-focus);
    outline-offset: 3px;
  }
  button:hover:not(:disabled) {
    background: var(--color-accent-wash);
  }
  input:not([type="checkbox"]),
  .field textarea,
  select {
    box-sizing: border-box;
    width: 100%;
    min-width: 0;
    border: 1px solid var(--color-control-border);
    border-radius: 6px;
    background: var(--color-control-bg);
    color: var(--color-control-text);
    font-size: var(--text-base);
    padding: 10px 11px;
  }
  input[type="checkbox"] {
    width: 16px;
    height: 16px;
    flex: 0 0 16px;
    accent-color: var(--color-accent);
    margin-top: 2px;
  }
  label {
    font-weight: 550;
  }
  small {
    display: block;
    font-size: var(--text-eyebrow);
    color: var(--color-text-muted);
    line-height: 1.6;
    font-weight: 400;
  }
  .eyebrow {
    font-size: var(--text-eyebrow);
    letter-spacing: 0.1em;
    color: var(--color-text-muted);
    font-weight: 600;
    text-transform: uppercase;
  }
  .workspace-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 21px 28px;
    border-bottom: 1px solid var(--color-border);
    gap: 16px;
    flex-shrink: 0;
  }
  .identity,
  .header-actions,
  .inline {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .export-icon {
    color: var(--color-accent-text);
    width: 44px;
    height: 44px;
    background: var(--color-accent-wash);
    display: grid;
    place-items: center;
    border-radius: 10px;
  }
  h1 {
    display: flex;
    gap: 14px;
    align-items: center;
    font-family: var(--font-display);
    font-size: var(--text-h2);
    margin: 0;
    font-weight: 500;
  }
  .quiet {
    padding: 8px;
    color: var(--color-text-muted);
    border: 0;
    background: transparent;
    font-size: var(--text-eyebrow);
  }
  .icon-button {
    width: 32px;
    height: 32px;
    padding: 5px;
    flex-shrink: 0;
    background: transparent;
    border: 0;
    color: var(--color-text-muted);
  }
  .duplicate-profile {
    border: 1px solid var(--color-border);
    width: 32px;
    height: 32px;
  }
  .profile-bar select {
    height: 32px;
    padding: 0 28px 0 10px;
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='16' fill='none' stroke='%23847c72' stroke-width='1.5'%3E%3Cpath d='m5 6 3 3 3-3'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 8px center;
  }
  .profile-bar {
    display: flex;
    align-items: end;
    gap: 24px;
    padding: 15px 28px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg);
    flex-shrink: 0;
  }
  .profile-bar label {
    display: block;
    font-size: var(--text-eyebrow);
    margin-bottom: 6px;
    color: var(--color-text-muted);
  }
  .profile-picker {
    width: 310px;
  }
  .profile-picker .inline {
    gap: 8px;
  }
  .format-picker {
    width: 240px;
  }
  .profile-status {
    margin-left: auto;
    padding-bottom: 2px;
    text-align: right;
  }
  .profile-status > span {
    display: flex;
    align-items: center;
    gap: 6px;
    justify-content: end;
    font-size: var(--text-eyebrow);
    color: var(--color-success);
  }
  .profile-status .unsaved {
    color: var(--color-text-muted);
  }
  .status-dot {
    width: 5px;
    height: 5px;
    background: var(--color-accent);
    border-radius: 50%;
  }
  .workspace-body {
    display: grid;
    grid-template-columns: 215px minmax(300px, 0.92fr) minmax(330px, 1.2fr);
    flex: 1;
    min-height: 0;
  }
  .navigation {
    display: flex;
    flex-direction: column;
    gap: 22px;
    padding: 22px 13px;
    background: var(--color-bg);
    border-right: 1px solid var(--color-border);
    overflow-y: auto;
  }
  .search-box {
    display: flex;
    align-items: center;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding-left: 9px;
    color: var(--color-text-muted);
    background: var(--color-surface);
  }
  .search-box input {
    border: 0;
    background: transparent;
    padding: 9px 6px;
    font-size: var(--text-base);
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  nav button {
    justify-content: start;
    text-align: left;
    width: 100%;
    border: 0;
    padding: 12px 10px;
    background: transparent;
    color: var(--color-text-muted);
    gap: 10px;
  }
  nav button span {
    flex: 1;
  }
  nav button small {
    font-size: var(--text-eyebrow);
    margin-top: 3px;
  }
  nav button.active {
    color: var(--color-accent-text);
    background: var(--color-accent-wash);
    box-shadow: inset 2px 0 var(--color-accent);
  }
  .sidebar-note {
    display: flex;
    gap: 10px;
    margin: auto 9px 0;
    padding-top: 32px;
    color: var(--color-text-muted);
  }
  .sidebar-note p {
    font-size: var(--text-eyebrow);
    line-height: 1.7;
    margin: 0;
  }
  .sidebar-note span {
    font-size: var(--text-eyebrow);
  }
  .search-results button {
    text-align: left;
    justify-content: space-between;
    padding: 12px 4px;
    width: 100%;
    border: 0;
    border-bottom: 1px solid var(--color-border);
    border-radius: 0;
    background: transparent;
    color: var(--color-text);
    font-size: var(--text-eyebrow);
  }
  .search-results p {
    font-size: var(--text-eyebrow);
    line-height: 1.6;
  }
  .settings-panel {
    padding: 28px;
    overflow-y: auto;
    min-width: 0;
  }
  .section-heading {
    margin-bottom: 27px;
  }
  h2 {
    font: 500 var(--text-h2) var(--font-display);
    margin: 7px 0;
    letter-spacing: -0.025em;
  }
  .section-heading p {
    margin: 0;
    color: var(--color-text-muted);
    line-height: 1.6;
    font-size: var(--text-eyebrow);
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 7px;
    margin: 0 0 20px;
  }
  .field-grid {
    display: grid;
    grid-template-columns: 1.5fr 1fr;
    gap: 14px;
  }
  .subheading {
    font-size: var(--text-eyebrow);
    font-weight: 600;
    margin: 28px 0 13px;
  }
  .preset-list {
    display: grid;
    gap: 9px;
  }
  .preset-list button {
    text-align: left;
    justify-content: start;
    gap: 13px;
    padding: 15px 12px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    color: var(--color-text);
  }
  .preset-list button.selected {
    border-color: var(--color-accent);
    background: var(--color-accent-wash);
  }
  .preset-list button > span:nth-child(2) {
    flex: 1;
  }
  .preset-list strong {
    display: block;
    font-weight: 550;
    margin-bottom: 3px;
  }
  .preset-icon {
    color: var(--color-accent-text);
    display: inline-flex;
  }
  .summary-list {
    border: 1px solid var(--color-border);
    border-radius: 7px;
    overflow: hidden;
  }
  .summary-list button {
    display: flex;
    justify-content: space-between;
    width: 100%;
    padding: 12px;
    font-size: var(--text-eyebrow);
    border: 0;
    background: transparent;
    color: var(--color-text-muted);
    border-bottom: 1px solid var(--color-border);
    border-radius: 0;
  }
  .summary-list button:last-child {
    border: 0;
  }
  .summary-list strong {
    flex: 1;
    text-align: right;
    font-weight: 400;
    color: var(--color-text);
  }
  .callout {
    background: var(--color-bg);
    border-left: 2px solid var(--color-border);
    padding: 13px 14px;
    line-height: 1.75;
    color: var(--color-text-muted);
    font-size: var(--text-eyebrow);
    margin-top: 24px;
  }
  .footnote {
    font-size: var(--text-eyebrow);
    color: var(--color-text-muted);
    line-height: 1.7;
    margin: 20px 0;
  }
  .check-row {
    display: flex;
    gap: 10px;
    padding: 12px 0;
    font-size: var(--text-eyebrow);
    cursor: pointer;
  }
  .check-row small {
    margin-top: 3px;
  }
  .chapter-list {
    max-height: 235px;
    overflow-y: auto;
    border: 1px solid var(--color-border);
    border-radius: 7px;
    padding: 4px 12px;
  }
  .chapter-row {
    display: flex;
    align-items: start;
    gap: 10px;
    padding: 10px 0;
    border-bottom: 1px solid var(--color-border);
    font-size: var(--text-eyebrow);
  }
  .chapter-row:last-child {
    border: 0;
  }
  .chip-row {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .chip-row button {
    padding: 5px 10px;
    background: var(--color-bg);
    color: var(--color-text-muted);
    border: 1px solid var(--color-border);
    font-size: var(--text-eyebrow);
  }
  .chip-row button.chosen {
    border-color: var(--color-accent);
    color: var(--color-accent-text);
  }
  details {
    border-top: 1px solid var(--color-border);
    border-bottom: 1px solid var(--color-border);
    margin: 20px 0;
  }
  summary {
    cursor: pointer;
    padding: 14px 0;
    font-size: var(--text-eyebrow);
  }
  summary span {
    color: var(--color-text-muted);
    float: right;
    font-size: var(--text-eyebrow);
  }
  details .field {
    margin-top: 8px;
  }
  .file-card {
    display: flex;
    align-items: center;
    gap: 13px;
    padding: 18px;
    border: 1px solid var(--color-border);
    border-radius: 7px;
    margin-top: 25px;
  }
  .file-card > div {
    flex: 1;
  }
  .file-card strong {
    font-size: var(--text-eyebrow);
    font-weight: 550;
  }
  .file-card > span {
    font: var(--text-eyebrow) var(--font-mono);
    color: var(--color-text-muted);
  }
  .filename-sample {
    overflow-wrap: anywhere;
    font-family: var(--font-mono);
  }
  .preview-panel {
    background: var(--color-surface-sunken);
    display: flex;
    flex-direction: column;
    border-left: 1px solid var(--color-border);
    min-width: 0;
    min-height: 0;
  }
  .preview-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 18px 20px 12px;
    flex-shrink: 0;
  }
  .preview-toolbar > div:first-child {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .live-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--color-success);
  }
  .preview-tabs {
    display: flex;
    gap: 2px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 2px;
  }
  .preview-tabs button {
    background: transparent;
    color: var(--color-text-muted);
    padding: 5px 9px;
    border: 0;
    font-size: var(--text-eyebrow);
  }
  .preview-tabs button.active {
    background: var(--color-surface);
    color: var(--color-text);
    box-shadow: var(--shadow-sm);
  }
  .preview-selection {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 20px 14px;
  }
  .preview-selection select {
    background: var(--color-surface);
    font-size: var(--text-eyebrow);
    padding: 7px 9px;
  }
  iframe {
    border: 0;
    width: 100%;
    flex: 1;
    min-height: 0;
    background: #e9e4da;
  }
  .source-preview {
    border: 0;
    resize: none;
    min-height: 0;
    background: transparent;
    flex: 1;
    margin: 0;
    padding: 22px;
    overflow: auto;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    font: var(--text-eyebrow)/1.8 var(--font-mono);
    color: var(--color-text);
  }
  .preview-caption {
    padding: 11px 20px;
    border-top: 1px solid var(--color-border);
    display: flex;
    justify-content: space-between;
    gap: 10px;
    font-size: var(--text-eyebrow);
    color: var(--color-text-muted);
    flex-shrink: 0;
  }
  .empty-preview {
    display: flex;
    flex: 1;
    align-items: center;
    justify-content: center;
    flex-direction: column;
    text-align: center;
    padding: 25px;
    color: var(--color-text-muted);
  }
  .empty-preview h3 {
    font: var(--text-h3) var(--font-display);
    color: var(--color-text);
  }
  .empty-preview p {
    font-size: var(--text-eyebrow);
    line-height: 1.6;
    max-width: 260px;
  }
  .empty-preview button {
    border: 1px solid var(--color-border);
    color: var(--color-text);
    background: var(--color-surface);
    padding: 9px 14px;
  }
  .workspace-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-top: 1px solid var(--color-border);
    padding: 15px 24px;
    flex-shrink: 0;
    gap: 12px;
  }
  .footer-note {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: var(--text-eyebrow);
    line-height: 1.7;
    color: var(--color-text-muted);
  }
  .footer-note small {
    font-size: var(--text-eyebrow);
  }
  .footer-actions {
    display: flex;
    gap: 10px;
    align-items: center;
  }
  .secondary,
  .primary {
    padding: 11px 15px;
    font-size: var(--text-eyebrow);
    font-weight: 550;
    border: 1px solid var(--color-border);
    color: var(--color-text);
    background: var(--color-surface);
  }
  .primary {
    background: var(--color-accent);
    color: var(--color-on-accent);
    border-color: var(--color-accent);
  }
  .primary:hover:not(:disabled) {
    background: var(--color-accent-text);
  }
  .primary:disabled,
  .secondary:disabled {
    background: var(--color-disabled-bg);
    color: var(--color-disabled-text);
    border-color: var(--color-disabled-border);
  }
  .feedback {
    display: flex;
    gap: 12px;
    align-items: center;
    justify-content: space-between;
    padding: 10px 24px;
    background: var(--color-success-wash);
    color: var(--color-success);
    font-size: var(--text-eyebrow);
    flex-shrink: 0;
  }
  .feedback.error {
    background: var(--color-error-wash);
    color: var(--color-error);
  }
  .text-button {
    display: inline-flex;
    border: 0;
    padding: 0;
    background: transparent;
    color: inherit;
    text-decoration: underline;
    font-size: inherit;
  }
  input:not([type="checkbox"]),
  .field textarea,
  select {
    font-size: var(--text-base);
  }
  input:not([type="checkbox"]):hover:not(:disabled),
  .field textarea:hover:not(:disabled),
  select:hover:not(:disabled) {
    border-color: var(--color-control-border-hover);
  }
  input:disabled,
  .field textarea:disabled,
  select:disabled {
    background: var(--color-disabled-bg);
    color: var(--color-disabled-text);
    border-color: var(--color-disabled-border);
  }
  .mobile-switch {
    display: none;
  }
  :global(.spinning) {
    animation: export-spin 1.5s linear infinite;
  }
  @keyframes export-spin {
    to {
      transform: rotate(360deg);
    }
  }
  @media (max-width: 1150px) {
    .workspace-body {
      grid-template-columns: 180px minmax(290px, 1fr) minmax(310px, 1fr);
    }
    .navigation {
      padding: 18px 8px;
    }
    nav button small {
      display: none;
    }
    .settings-panel {
      padding: 24px 20px;
    }
    .profile-bar {
      gap: 16px;
    }
  }
  @media (max-width: 900px) {
    .export-workspace {
      width: calc(100vw - 16px);
      height: calc(100vh - 16px);
    }
    .workspace-header {
      padding: 15px 18px;
    }
    h1 {
      font-size: var(--text-h2);
    }
    .export-icon {
      display: none;
    }
    .profile-bar {
      padding: 12px 18px;
    }
    .profile-status {
      display: none;
    }
    .profile-picker,
    .format-picker {
      flex: 1;
      width: auto;
    }
    input:not([type="checkbox"]),
    .field textarea,
    select {
      font-size: var(--text-base);
    }
    input:not([type="checkbox"]):hover:not(:disabled),
    .field textarea:hover:not(:disabled),
    select:hover:not(:disabled) {
      border-color: var(--color-control-border-hover);
    }
    input:disabled,
    .field textarea:disabled,
    select:disabled {
      background: var(--color-disabled-bg);
      color: var(--color-disabled-text);
      border-color: var(--color-disabled-border);
    }
    .mobile-switch {
      display: flex;
      border-bottom: 1px solid var(--color-border);
      padding: 6px 18px;
      gap: 8px;
    }
    .mobile-switch button {
      padding: 8px 16px;
      border: 0;
      background: transparent;
      color: var(--color-text-muted);
    }
    .mobile-switch button.active {
      background: var(--color-accent-wash);
      color: var(--color-accent-text);
    }
    .workspace-body {
      grid-template-columns: 175px minmax(0, 1fr);
    }
    .preview-panel {
      grid-column: 1/-1;
    }
    .workspace-body .mobile-hidden {
      display: none;
    }
    .footer-note {
      display: none;
    }
    .workspace-footer {
      justify-content: end;
      padding: 12px;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    :global(.spinning) {
      animation: none;
    }
    button {
      transition: none;
    }
  }
</style>
