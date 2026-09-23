<!--
  CommandPalette.svelte - Cmd+K command palette (US-1.1-5)

  Fuzzy-searchable list of commands with keyboard shortcuts.
  Displays the current configured bindings.
-->
<script lang="ts">
  import { shortcuts } from "../stores/shortcuts.svelte";
  import { Search } from "lucide-svelte";
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
      selectedIndex = filteredCommands.length ? (selectedIndex + 1) % filteredCommands.length : 0;
      return;
    }

    if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = filteredCommands.length
        ? (selectedIndex - 1 + filteredCommands.length) % filteredCommands.length
        : 0;
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
  <!-- The shared modal scrim: no blur. -->
  <div
    class="palette-scrim"
    role="presentation"
    aria-hidden="true"
    onclick={() => (open = false)}
  ></div>

  <dialog
    class="ka-dialog palette"
    open
    aria-modal="true"
    aria-label="Command palette"
    data-testid="command-palette"
  >
    <div class="palette-search">
      <Search class="w-5 h-5 ka-icon" aria-hidden="true" />
      <!-- svelte-ignore a11y_autofocus -->
      <input
        type="text"
        placeholder="Type a command or search…"
        aria-label="Search commands"
        aria-controls="command-palette-list"
        aria-activedescendant={filteredCommands.length
          ? `command-palette-item-${selectedIndex}`
          : undefined}
        bind:value={query}
        autofocus
      />
      {#if shortcuts.label("command_palette")}<kbd>{shortcuts.label("command_palette")}</kbd>{/if}
    </div>

    <div
      class="ka-command-list palette-list"
      id="command-palette-list"
      role="listbox"
      aria-label="Commands"
    >
      {#if filteredCommands.length === 0}
        <div class="ka-empty od-stack palette-empty">
          <h4>No matching commands</h4>
          <p>Check the spelling, or try a shorter word.</p>
        </div>
      {:else}
        {#each filteredCommands as cmd, i}
          <button
            id="command-palette-item-{i}"
            type="button"
            role="option"
            aria-selected={i === selectedIndex}
            onclick={() => runCommand(cmd)}
            class:ka-command-selected={i === selectedIndex}
          >
            <span class="palette-label">
              <span>{cmd.label}</span>
              <small>{cmd.category}</small>
            </span>
            {#if cmd.shortcut}<kbd>{cmd.shortcut}</kbd>{/if}
          </button>
        {/each}
      {/if}
    </div>

    <p class="palette-hints">
      <span><kbd>↑</kbd> <kbd>↓</kbd> to navigate</span>
      <span><kbd>Enter</kbd> to run</span>
      <span><kbd>Esc</kbd> to close</span>
    </p>
  </dialog>
{/if}

<style>
  .palette-scrim {
    position: fixed;
    inset: 0;
    z-index: var(--z-command-backdrop);
    background: var(--color-overlay-scrim);
  }
  .palette {
    z-index: var(--z-command);
    top: 15dvh;
    bottom: auto;
    margin: 0 auto;
    width: min(600px, calc(100% - 32px));
  }
  .palette-search {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-s);
    border-bottom: var(--border-hair);
    color: var(--color-text-muted);
  }
  .palette-search input {
    flex: 1;
    min-width: 0;
    min-height: var(--control-target);
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--color-text);
    font: var(--text-base) / 1.5 var(--font-ui);
  }
  .palette-search input:focus-visible {
    outline: none;
  }
  .palette-search:has(input:focus-visible) {
    outline: 2px solid var(--color-accent-text);
    outline-offset: -2px;
    border-radius: var(--ka-radius) var(--ka-radius) 0 0;
  }
  .palette-list {
    margin: 0;
    padding: var(--space-2xs);
    max-height: 360px;
  }
  .palette-list :global(button) {
    padding-block: var(--space-2xs);
  }
  .palette-label {
    display: grid;
    gap: 2px;
    min-width: 0;
  }
  .palette-label > span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .palette-empty {
    align-items: center;
    padding: var(--space-l) var(--space-s);
    text-align: center;
  }
  .palette-empty h4 {
    margin: 0;
    font: 550 var(--text-h3) / 1.25 var(--font-display);
    color: var(--color-text);
  }
  .palette-hints {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-s);
    margin: 0;
    padding: var(--space-xs) var(--space-s);
    border-top: var(--border-hair);
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .palette kbd {
    font: var(--text-small) / 1.4 var(--font-mono);
    color: var(--color-text-muted);
    white-space: nowrap;
  }
</style>
