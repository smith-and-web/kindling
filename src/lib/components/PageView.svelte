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

<section>
  <div class="flex items-center justify-between mb-4">
    <h2 class="text-press-ui font-semibold text-press-text uppercase tracking-wide">Scene Prose</h2>
    <span class="text-press-eyebrow text-press-muted">{wordCount} words</span>
  </div>
  <div class="bg-press-surface rounded-lg overflow-hidden" style="min-height: 50rem;">
    <NovelEditor
      {projectId}
      {sceneId}
      {content}
      placeholder={readonly ? "Scene is locked" : "Write your scene prose here..."}
      {readonly}
      {saveStatus}
      {onUpdate}
    />
  </div>
</section>
