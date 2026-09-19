<script lang="ts">
  import ClassicExportDialog from "./ClassicExportDialog.svelte";
  import ExportWorkspace from "./ExportWorkspace.svelte";
  import type { ExportResult } from "../types";

  let {
    scope,
    scopeId,
    scopeTitle,
    onClose,
    onSuccess,
  }: {
    scope: "project" | "chapter" | "scene";
    scopeId: string | null;
    scopeTitle: string;
    onClose: () => void;
    onSuccess: (result: ExportResult) => void;
  } = $props();
  let classic = $state(true);
</script>

{#if classic}
  <ClassicExportDialog
    {scope}
    {scopeId}
    {scopeTitle}
    {onClose}
    {onSuccess}
    onCustomize={() => (classic = false)}
  />
{:else}
  <ExportWorkspace {scope} {scopeId} {onClose} onClassic={() => (classic = true)} />
{/if}
