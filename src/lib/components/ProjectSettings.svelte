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
  import { t } from "../i18n";

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
      error = t("settings.project.loadDailyGoalFailed", { error: String(e) });
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
        throw new Error(t("settings.project.wordTargetInvalid"));
      }

      if (
        goalLoaded &&
        (dailyGoal === undefined ||
          !Number.isInteger(dailyGoal) ||
          dailyGoal < 0 ||
          dailyGoal > 1000000)
      ) {
        throw new Error(t("settings.project.dailyGoalInvalid"));
      }

      const disabledDraft = REFERENCE_TYPE_OPTIONS.find(
        (option) => fieldStates[option.id]?.dirty && !referenceTypes.includes(option.id)
      );
      if (disabledDraft) {
        throw new Error(t("settings.project.disabledDraft", { label: disabledDraft.label }));
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

<div class="settings-pane" aria-busy={busy}>
  <p class="settings-lede">
    {t("settings.project.ledePrefix")}<strong>{project.name}</strong>{t(
      "settings.project.ledeSuffix"
    )}
    {#if section === "tags"}
      {t("settings.project.tagChanges")}
    {:else if section === "fields"}
      {t("settings.project.fieldChanges")}
    {/if}
  </p>
  <div hidden={section !== "details"} class="settings-fields">
    <!-- Pen Name -->
    <div class="ka-field od-field">
      <label for="author-pen-name">
        {t("settings.project.penName")}
        <span class="ka-optional">{t("settings.project.optional")}</span>
      </label>
      <input
        id="author-pen-name"
        type="text"
        bind:value={authorPenName}
        placeholder={t("settings.project.penNamePlaceholder")}
        disabled={saving}
      />
      <p class="ka-help">
        {t("settings.project.penNameHelp")}
      </p>
    </div>

    <!-- Genre -->
    <div class="ka-field od-field">
      <label for="genre">
        {t("settings.project.genre")}
        <span class="ka-optional">{t("settings.project.optional")}</span>
      </label>
      <input
        id="genre"
        type="text"
        bind:value={genre}
        placeholder={t("settings.project.genrePlaceholder")}
        disabled={saving}
      />
      <p class="ka-help">{t("settings.project.genreHelp")}</p>
    </div>

    <!-- Description -->
    <div class="ka-field od-field">
      <label for="project-description">
        {t("settings.project.description")}
        <span class="ka-optional">{t("settings.project.optional")}</span>
      </label>
      <textarea
        id="project-description"
        rows="4"
        bind:value={description}
        placeholder={t("settings.project.descriptionPlaceholder")}
        disabled={saving}
      ></textarea>
    </div>

    <div class="ka-field od-field">
      <label for="daily-writing-goal">{t("settings.project.dailyGoal")}</label>
      <input
        id="daily-writing-goal"
        type="number"
        min="0"
        max="1000000"
        step="1"
        bind:value={dailyGoal}
        disabled={saving || !goalLoaded}
        aria-describedby="daily-goal-help"
      />
      <p id="daily-goal-help" class="ka-help">
        {t("settings.project.dailyGoalHelp")}
      </p>
    </div>

    <!-- Word Target -->
    <div class="ka-field od-field">
      <label for="word-target">
        {t("settings.project.wordTarget")}
        <span class="ka-optional">{t("settings.project.optional")}</span>
      </label>
      <input
        id="word-target"
        type="number"
        min="0"
        inputmode="numeric"
        bind:value={wordTarget}
        placeholder={t("settings.project.wordTargetPlaceholder")}
        disabled={saving}
      />
    </div>
  </div>
  <div hidden={section !== "references"} class="settings-fields">
    <p class="ka-help">
      {t("settings.project.referencesHelp")}
    </p>
    {#each REFERENCE_TYPE_OPTIONS as option (option.id)}
      <label class="ka-check">
        <input type="checkbox" value={option.id} bind:group={referenceTypes} disabled={saving} />
        {option.label}
      </label>
    {/each}
  </div>
  <div hidden={section !== "tags"}>
    {#if tagsVisited}
      <TagManager
        projectId={project.id}
        onState={(state) => (tagState = state)}
        onChange={() => onSave({ ...project })}
      />
    {/if}
  </div>
  <div hidden={section !== "fields"}>
    {#if fieldsVisited}
      {#each normalizeReferenceTypes(project.reference_types ?? DEFAULT_REFERENCE_TYPES) as refType (refType)}
        {@const mapping = REFERENCE_TYPE_OPTIONS.find((option) => option.id === refType)}
        {#if mapping}<FieldDefinitionManager
            projectId={project.id}
            entityType={REFERENCE_FIELD_TYPES[refType]}
            entityLabel={mapping.label}
            onState={(state) => (fieldStates[refType] = state)}
            onChange={() => onSave({ ...project })}
          />{/if}
      {:else}<p class="ka-help">{t("settings.project.enableRefTypes")}</p>{/each}
    {/if}
  </div>
  {#if section === "details" || section === "references"}
    {#if error}<p role="alert" class="ka-error">{error}</p>{/if}
    <div class="settings-actions">
      {#if saved && !dirty}<p role="status" class="ka-help">{t("settings.project.saved")}</p>{/if}
      <button
        type="button"
        onclick={handleSave}
        disabled={saving}
        aria-busy={saving || undefined}
        class="ka-button"
        >{#if saving}<Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
          {t("common.saving")}{:else}{t("settings.project.save")}{/if}</button
      >
    </div>
  {/if}
</div>
