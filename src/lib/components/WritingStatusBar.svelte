<script lang="ts">
  import { currentProject } from "../stores/project.svelte";
  import { writing } from "../stores/writing.svelte";

  let expanded = $state(false);
  const stats = $derived(
    writing.value?.project_id === currentProject.value?.id ? writing.value : null
  );
  const sceneId = $derived(currentProject.currentScene?.id);
  const chapterId = $derived(
    currentProject.currentScene?.chapter_id ?? currentProject.currentChapter?.id
  );
  const sceneCounts = $derived(Object.values(stats?.scene_words ?? {}));
  const scenesWithProse = $derived(sceneCounts.filter((words) => words > 0).length);
  const averageWords = $derived(
    sceneCounts.length ? (stats?.project_words ?? 0) / sceneCounts.length : 0
  );
  const chapters = $derived(
    currentProject.chapters.filter(
      (chapter) =>
        !chapter.archived && !chapter.is_part && stats?.chapter_words[chapter.id] !== undefined
    )
  );

  function toggleStatistics() {
    expanded = !expanded;
    if (expanded) void writing.refresh();
  }
</script>

{#if currentProject.value && (stats || writing.error)}
  <footer class="statusbar">
    {#if writing.error}
      <p role="alert" class="statusbar-error">{writing.error}</p>
    {/if}
    {#if stats}
      {#if expanded}
        <!-- The scrollable panel needs a tab stop for keyboard scrolling. -->
        <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
        <section
          id="writing-statistics-panel"
          aria-labelledby="writing-statistics-title"
          class="statistics-panel"
          tabindex="0"
        >
          <h2 id="writing-statistics-title">Writing statistics</h2>
          <dl class="ka-stats statistics-figures">
            <div>
              <dt>Total words</dt>
              <dd>{stats.project_words.toLocaleString()}</dd>
            </div>
            <div>
              <dt>Scenes with prose</dt>
              <dd>{scenesWithProse.toLocaleString()}</dd>
            </div>
            <div>
              <dt>Empty scenes</dt>
              <dd>
                {(sceneCounts.length - scenesWithProse).toLocaleString()}
              </dd>
            </div>
            <div>
              <dt>Average words per scene</dt>
              <dd>
                {averageWords.toLocaleString(undefined, { maximumFractionDigits: 1 })}
              </dd>
            </div>
          </dl>
          <p class="ka-help statistics-note">
            Saved prose only. Archived content is excluded; the average includes empty scenes.
          </p>
          {#if chapters.length}
            <table class="statistics-table">
              <caption>Words per chapter</caption>
              <thead>
                <tr>
                  <th scope="col">Chapter</th>
                  <th scope="col">Words</th>
                </tr>
              </thead>
              <tbody>
                {#each chapters as chapter (chapter.id)}
                  <tr>
                    <th scope="row">
                      {chapter.title}
                    </th>
                    <td>
                      {stats.chapter_words[chapter.id].toLocaleString()}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {:else}
            <p class="ka-help">No chapters yet.</p>
          {/if}
        </section>
      {/if}
      <div class="statusbar-row">
        <span
          >Scene: {sceneId ? (stats.scene_words[sceneId]?.toLocaleString() ?? "—") : "—"} words</span
        >
        <span class="statusbar-chapter"
          >Chapter: {chapterId ? (stats.chapter_words[chapterId]?.toLocaleString() ?? "—") : "—"} words</span
        >
        <span class="statusbar-project">Project: {stats.project_words.toLocaleString()} words</span>
        <span title="Net words saved since opening the app or resetting the session.">
          Session: {stats.session_words.toLocaleString()} words
        </span>
        <button
          type="button"
          class="ka-button ka-button--ghost statusbar-toggle"
          aria-expanded={expanded}
          aria-controls="writing-statistics-panel"
          onclick={toggleStatistics}>{expanded ? "Hide statistics" : "Writing statistics"}</button
        >
      </div>
    {/if}
  </footer>
{/if}

<style>
  .statusbar {
    flex: none;
    border-top: var(--border-hair);
    background: var(--color-surface);
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .statusbar-error {
    margin: 0;
    padding: var(--space-2xs) var(--space-m);
    color: var(--color-error);
  }
  .statusbar-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    column-gap: var(--space-m);
    row-gap: 0;
    min-height: var(--control-target);
    padding: 0 var(--space-3xs) 0 var(--space-m);
    font-variant-numeric: tabular-nums;
  }
  .statusbar-row > span {
    white-space: nowrap;
  }
  .statusbar-toggle {
    margin-left: auto;
    font-size: var(--text-small);
  }
  .statistics-panel:focus-visible {
    outline: 2px solid var(--color-accent-text);
    outline-offset: -2px;
  }
  /* Compact windows keep one row; chapter and project totals stay in the
     statistics panel. */
  @media (max-width: 1280px) {
    .statusbar-row {
      flex-wrap: nowrap;
      column-gap: var(--space-s);
    }
    .statusbar-chapter,
    .statusbar-project {
      display: none;
    }
  }
  .statistics-panel {
    max-height: 35vh;
    overflow-y: auto;
    padding: var(--space-s) var(--space-m);
    border-bottom: var(--border-hair);
    color: var(--color-text);
  }
  .statistics-panel h2 {
    margin: 0;
    font: 550 var(--text-h3) / var(--leading-tight) var(--font-display);
    letter-spacing: var(--tracking-tight);
  }
  .statistics-figures {
    margin: var(--space-s) 0 var(--space-2xs);
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  }
  /* A label that wraps keeps its value on the shared baseline row. */
  .statistics-figures > :global(div) {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }
  /* Short windows: room for the chapter breakdown under the totals. */
  @media (max-height: 800px) {
    .statistics-panel {
      max-height: 50vh;
    }
  }
  .statistics-note {
    margin: 0 0 var(--space-s);
  }
  .statistics-table {
    width: 100%;
    table-layout: fixed;
    border-collapse: collapse;
    font: var(--text-small) / 1.5 var(--font-ui);
  }
  .statistics-table caption {
    margin-bottom: var(--space-2xs);
    font-weight: 500;
    text-align: left;
  }
  .statistics-table :where(th, td) {
    padding: var(--space-2xs) 0;
    border-bottom: var(--border-hair);
    text-align: left;
    font-weight: 400;
    overflow-wrap: anywhere;
  }
  .statistics-table thead th {
    font-weight: 500;
    color: var(--color-text-muted);
  }
  .statistics-table :where(td, thead th:last-child) {
    width: 6rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
</style>
