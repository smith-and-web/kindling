<!--
  CommandPalette.svelte - Cmd+K command palette (US-1.1-5)

  Fuzzy-searchable list of commands with keyboard shortcuts.
  Each command has an accompanying shortcut; palette opens with ⌘K.
-->
<script lang="ts">
  import { Command, Search } from "lucide-svelte";
  import { fuzzyMatch, fuzzyScore, type CommandDef } from "../commands";

  interface CommandWithAction extends CommandDef {
    action: () => void;
  }

  let {
    open = $bindable(false),
    commands,
    onClose,
  }: {
    open?: boolean;
    commands: CommandWithAction[];
    onClose?: () => void;
  } = $props();

  let query = $state("");
  // eslint-disable-next-line svelte/prefer-writable-derived -- mutated by arrow key navigation
  let selectedIndex = $state(0);

  const filteredCommands = $derived.by(() => {
    const q = query.trim();
    if (!q) return commands;
    return commands
      .filter((c) => {
        const searchText = [c.label, c.keywords?.join(" ") ?? ""].join(" ").toLowerCase();
        return fuzzyMatch(q, searchText);
      })
      .sort((a, b) => {
        const scoreA = fuzzyScore(q, a.label + " " + (a.keywords?.join(" ") ?? ""));
        const scoreB = fuzzyScore(q, b.label + " " + (b.keywords?.join(" ") ?? ""));
        return scoreB - scoreA;
      });
  });

  $effect(() => {
    selectedIndex = Math.min(selectedIndex, Math.max(0, filteredCommands.length - 1));
  });

  $effect(() => {
    if (open) {
      query = "";
      selectedIndex = 0;
    }
  });

  // Scroll selected item into view when navigating with arrows
  $effect(() => {
    if (!open || filteredCommands.length === 0) return;
    const el = document.getElementById(`command-palette-item-${selectedIndex}`);
    el?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  });

  function handleKeydown(e: KeyboardEvent) {
    if (!open) return;

    if (e.key === "Escape") {
      e.preventDefault();
      onClose?.();
      open = false;
      return;
    }

    if (e.key === "Enter") {
      e.preventDefault();
      const cmd = filteredCommands[selectedIndex];
      if (cmd) {
        cmd.action();
        open = false;
        onClose?.();
      }
      return;
    }

    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % filteredCommands.length;
      return;
    }

    if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + filteredCommands.length) % filteredCommands.length;
      return;
    }

    // Don't capture alphanumeric/space - let input handle it
    if (e.key.length === 1 && !e.metaKey && !e.ctrlKey && !e.altKey) {
      return;
    }
  }

  function runCommand(cmd: CommandWithAction) {
    cmd.action();
    open = false;
    onClose?.();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 z-[110] bg-black/30 backdrop-blur-sm"
    role="presentation"
    aria-hidden="true"
    onclick={() => (open = false)}
  ></div>

  <!-- Palette -->
  <div
    class="fixed left-1/2 top-[20%] z-[111] w-[min(32rem,90vw)] -translate-x-1/2 rounded-xl border border-bg-card bg-bg-panel shadow-[0_8px_32px_rgba(0,0,0,0.4)]"
    role="dialog"
    aria-modal="true"
    aria-label="Command palette"
  >
    <!-- Search input -->
    <div class="flex items-center gap-2 border-b border-bg-card px-4 py-3">
      <Search class="w-4 h-4 shrink-0 text-text-secondary" />
      <!-- svelte-ignore a11y_autofocus -->
      <input
        type="text"
        placeholder="Type a command or search..."
        bind:value={query}
        class="flex-1 bg-transparent text-text-primary placeholder:text-text-secondary focus:outline-none"
        autofocus
      />
      <kbd class="rounded border border-bg-card px-2 py-0.5 text-xs text-text-secondary">⌘K</kbd>
    </div>

    <!-- Command list -->
    <div class="max-h-80 overflow-y-auto py-2">
      {#if filteredCommands.length === 0}
        <p class="px-4 py-8 text-center text-sm text-text-secondary">No matching commands</p>
      {:else}
        {#each filteredCommands as cmd, i}
          <button
            id="command-palette-item-{i}"
            type="button"
            onclick={() => runCommand(cmd)}
            class="flex w-full items-center justify-between gap-4 px-4 py-2.5 text-left transition-colors {i ===
            selectedIndex
              ? 'bg-accent/15'
              : 'hover:bg-accent/10'}"
          >
            <div class="flex items-center gap-3 min-w-0">
              <Command class="w-4 h-4 shrink-0 text-text-secondary" />
              <span class="truncate text-text-primary">{cmd.label}</span>
            </div>
            <kbd
              class="shrink-0 rounded border border-bg-card px-2 py-0.5 text-xs text-text-secondary"
            >
              {cmd.shortcut}
            </kbd>
          </button>
        {/each}
      {/if}
    </div>

    <p class="border-t border-bg-card px-4 py-2 text-xs text-text-secondary">
      ↑↓ to navigate · Enter to run · Esc to close
    </p>
  </div>
{/if}
