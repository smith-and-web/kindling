<script lang="ts">
  import NovelEditor from "./NovelEditor.svelte";

  let {
    content,
    readonly = false,
    saveStatus = "idle",
    wordCount = 0,
    onUpdate,
    projectId,
    sceneId,
  }: {
    content: string;
    readonly?: boolean;
    saveStatus?: "idle" | "saving" | "error";
    wordCount?: number;
    onUpdate: (html: string) => void;
    projectId?: string;
    sceneId?: string;
  } = $props();
</script>

<section class="page-view" aria-labelledby="scene-prose-title">
  <div class="page-view-head">
    <h3 id="scene-prose-title">Scene prose</h3>
    <span class="page-view-count">{wordCount} words</span>
  </div>
  <div class="page-view-body">
    <NovelEditor
      {projectId}
      {sceneId}
      {content}
      placeholder={readonly ? "Scene is locked" : "Write your scene prose here…"}
      {readonly}
      {saveStatus}
      {onUpdate}
    />
  </div>
</section>

<style>
  .page-view {
    margin-top: var(--space-m);
    padding-top: var(--space-s);
    border-top: var(--border-hair);
  }
  .page-view-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-s);
    min-height: var(--control-target);
    margin-bottom: var(--space-2xs);
  }
  .page-view-head h3 {
    margin: 0;
    font: 550 var(--text-h3) / var(--leading-tight) var(--font-display);
    letter-spacing: var(--tracking-tight);
    color: var(--color-text);
  }
  .page-view-count {
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
  }
  .page-view-body {
    min-height: 50rem;
    overflow: hidden;
    border: var(--border-hair);
    border-radius: var(--radius-m);
  }
</style>
