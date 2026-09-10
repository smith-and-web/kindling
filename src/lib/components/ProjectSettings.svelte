<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { writing } from "../stores/writing.svelte";
  import type { WritingStats } from "../types";
  import { REFERENCE_FIELD_TYPES, REFERENCE_TYPE_OPTIONS } from "../referenceTypes";
  import { invoke } from "@tauri-apps/api/core";
  import { Loader2 } from "lucide-svelte";
  import type { Project } from "../types";
  import { normalizeReferenceTypes, DEFAULT_REFERENCE_TYPES } from "../referenceTypes";
  import FieldDefinitionManager from "./FieldDefinitionManager.svelte";
  import TagManager from "./TagManager.svelte";

  let {
    project,
    section,
    onSave,
    dirty = $bindable(false),
    busy = $bindable(false),
  }: {
    project: Project;
    section: string;
    onSave: (project: Project) => void;
    dirty?: boolean;
    busy?: boolean;
  } = $props();
  // The parent keys this form by project id; drafts survive section navigation.
  const initial = untrack(() => project);
  let baseline = $state("");
  let saved = $state(false);
  let referenceTypes = $state(
    normalizeReferenceTypes(initial.reference_types ?? DEFAULT_REFERENCE_TYPES)
  );
  // Form fields initialized from current project
  let authorPenName = $state(initial.author_pen_name ?? "");
  let genre = $state(initial.genre ?? "");
  let description = $state(initial.description ?? "");
  let wordTarget = $state(
    initial.word_target !== null && initial.word_target !== undefined
      ? String(initial.word_target)
      : ""
  );
  const projectId = initial.id;
  baseline = untrack(() =>
    JSON.stringify([authorPenName, genre, description, wordTarget, referenceTypes])
  );
  let dailyGoal = $state<number | undefined>(undefined);
  let goalLoaded = $state(false);
  onMount(async () => {
    try {
      const stats = await invoke<WritingStats>("get_writing_stats", { projectId });
      dailyGoal = stats.daily_goal;
      savedGoal = dailyGoal;
      goalLoaded = true;
    } catch (e) {
      error = `Could not load daily goal: ${String(e)}`;
    }
  });
  let saving = $state(false);
  let error = $state<string | null>(null);

  const snapshot = $derived(
    JSON.stringify([authorPenName, genre, description, wordTarget, referenceTypes])
  );
  let tagState = $state({ dirty: false, busy: false });
  let fieldStates = $state<Record<string, { dirty: boolean; busy: boolean }>>({});
  let tagsVisited = $state(false);
  let fieldsVisited = $state(false);
  $effect(() => {
    if (section === "tags") tagsVisited = true;
    if (section === "fields") fieldsVisited = true;
  });
  let savedGoal = $state<number | undefined>(undefined);
  $effect(() => {
    dirty =
      snapshot !== baseline ||
      (goalLoaded && dailyGoal !== savedGoal) ||
      tagState.dirty ||
      Object.values(fieldStates).some((state) => state.dirty);
    busy = saving || tagState.busy || Object.values(fieldStates).some((state) => state.busy);
  });

  async function handleSave() {
    if (saving) return;

    saving = true;
    saved = false;
    error = null;

    try {
      const parsedWordTarget = String(wordTarget ?? "").trim().length
        ? Number(String(wordTarget ?? "").trim())
        : null;
      if (
        parsedWordTarget !== null &&
        (!Number.isSafeInteger(parsedWordTarget) || parsedWordTarget < 0)
      ) {
        throw new Error("Word target must be a non-negative whole number");
      }

      if (
        goalLoaded &&
        (dailyGoal === undefined ||
          !Number.isInteger(dailyGoal) ||
          dailyGoal < 0 ||
          dailyGoal > 1000000)
      ) {
        throw new Error("Daily goal must be a whole number between 0 and 1,000,000");
      }

      const disabledDraft = REFERENCE_TYPE_OPTIONS.find(
        (option) => fieldStates[option.id]?.dirty && !referenceTypes.includes(option.id)
      );
      if (disabledDraft) {
        throw new Error(
          `Save or cancel the ${disabledDraft.label} custom field draft before disabling that reference type.`
        );
      }

      // Convert empty strings to null for optional fields
      const settings = {
        author_pen_name: authorPenName.trim() || null,
        genre: genre.trim() || null,
        description: description.trim() || null,
        word_target: parsedWordTarget,
        reference_types: referenceTypes,
        // This command replaces nullable metadata; preserve the screenplay target.
        target_page_count: project.target_page_count,
        ...(goalLoaded ? { daily_writing_goal: dailyGoal } : {}),
      };

      const updatedProject = await invoke<Project>("update_project_settings", {
        projectId,
        settings,
      });

      void writing.refresh(projectId);
      baseline = snapshot;
      savedGoal = dailyGoal;
      saved = true;
      onSave(updatedProject);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }
</script>

<div class="space-y-5" aria-busy={busy}>
  <p class="text-press-ui text-press-muted">
    Changes here apply only to <strong class="text-press-text">{project.name}</strong>.
  </p>
  <div hidden={section !== "details"} class="space-y-4">
    <!-- Pen Name -->
    <div>
      <label for="author-pen-name" class="block text-press-ui text-press-muted mb-1">
        Pen Name <span class="text-press-muted">(optional)</span>
      </label>
      <input
        id="author-pen-name"
        type="text"
        bind:value={authorPenName}
        placeholder="Leave blank to use your author name"
        disabled={saving}
        class="w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2 focus:outline-none focus:border-press-accent"
      />
      <p class="text-press-eyebrow text-press-muted mt-1">
        If provided, this will be used as the byline on title pages instead of your author name.
      </p>
    </div>

    <!-- Genre -->
    <div>
      <label for="genre" class="block text-press-ui text-press-muted mb-1">
        Genre <span class="text-press-muted">(optional)</span>
      </label>
      <input
        id="genre"
        type="text"
        bind:value={genre}
        placeholder="e.g., Literary Fiction, Science Fiction, Mystery"
        disabled={saving}
        class="w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2 focus:outline-none focus:border-press-accent"
      />
      <p class="text-press-eyebrow text-press-muted mt-1">
        Genre will be displayed on manuscript title pages.
      </p>
    </div>

    <!-- Description -->
    <div>
      <label for="project-description" class="block text-press-ui text-press-muted mb-1">
        Project Description <span class="text-press-muted">(optional)</span>
      </label>
      <textarea
        id="project-description"
        rows="4"
        bind:value={description}
        placeholder="Short summary or notes about this project"
        disabled={saving}
        class="w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2 focus:outline-none focus:border-press-accent resize-none"
      ></textarea>
    </div>

    <div>
      <label for="daily-writing-goal" class="block text-press-ui text-press-muted mb-1"
        >Daily writing goal</label
      >
      <input
        id="daily-writing-goal"
        type="number"
        min="0"
        max="1000000"
        step="1"
        bind:value={dailyGoal}
        disabled={saving || !goalLoaded}
        class="w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2"
        aria-describedby="daily-goal-help"
      />
      <p id="daily-goal-help" class="text-press-eyebrow text-press-muted mt-1">
        Net words added per day in this project. Set to 0 to turn off the goal. Changes apply today;
        earlier streak days keep their original goals.
      </p>
    </div>

    <!-- Word Target -->
    <div>
      <label for="word-target" class="block text-press-ui text-press-muted mb-1">
        Word Target <span class="text-press-muted">(optional)</span>
      </label>
      <input
        id="word-target"
        type="number"
        min="0"
        inputmode="numeric"
        bind:value={wordTarget}
        placeholder="e.g., 80000"
        disabled={saving}
        class="w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2 focus:outline-none focus:border-press-accent"
      />
    </div>
  </div>
  <div hidden={section !== "references"} class="space-y-3">
    <p class="text-press-ui text-press-muted">
      Choose which reference types appear in this project’s References panel. Disabling a type keeps
      its existing entries.
    </p>
    {#each REFERENCE_TYPE_OPTIONS as option (option.id)}
      <label class="flex items-center gap-2 text-press-ui text-press-text">
        <input type="checkbox" value={option.id} bind:group={referenceTypes} disabled={saving} />
        {option.label}
      </label>
    {/each}
  </div>
  <div hidden={section !== "tags"}>
    {#if tagsVisited}
      <p class="text-press-ui text-press-muted">Tag changes save as you make them.</p>
      <TagManager
        projectId={project.id}
        onState={(state) => (tagState = state)}
        onChange={() => onSave({ ...project })}
      />
    {/if}
  </div>
  <div hidden={section !== "fields"}>
    {#if fieldsVisited}
      <p class="text-press-ui text-press-muted">
        Custom field changes save as you make them. Fields are listed for saved, enabled reference
        types.
      </p>
      {#each normalizeReferenceTypes(project.reference_types ?? DEFAULT_REFERENCE_TYPES) as refType (refType)}
        {@const mapping = REFERENCE_TYPE_OPTIONS.find((option) => option.id === refType)}
        {#if mapping}<FieldDefinitionManager
            projectId={project.id}
            entityType={REFERENCE_FIELD_TYPES[refType]}
            entityLabel={mapping.label}
            onState={(state) => (fieldStates[refType] = state)}
            onChange={() => onSave({ ...project })}
          />{/if}
      {:else}<p class="text-press-ui text-press-muted">
          Enable reference types to configure custom fields.
        </p>{/each}
    {/if}
  </div>
  {#if section === "details" || section === "references"}
    {#if error}<p role="alert" class="text-press-ui text-press-error">{error}</p>{/if}
    <button
      type="button"
      onclick={handleSave}
      disabled={saving}
      class="px-4 py-2 bg-press-accent text-press-on-accent rounded-lg"
      >{#if saving}<Loader2 class="w-4 h-4 animate-spin" />{:else}Save project changes{/if}</button
    >
    {#if saved && !dirty}<p role="status" class="text-press-ui text-press-muted">
        Project changes saved.
      </p>{/if}
  {/if}
</div>
