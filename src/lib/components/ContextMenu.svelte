<script lang="ts">
  /* eslint-disable no-undef */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  type IconComponent = any;

  interface MenuItem {
    label: string;
    icon?: IconComponent;
    action: () => void | Promise<void>;
    disabled?: boolean;
    danger?: boolean;
    divider?: boolean;
  }

  let {
    items,
    x,
    y,
    onClose,
  }: {
    items: MenuItem[];
    x: number;
    y: number;
    onClose: () => void;
  } = $props();

  let menuRef: HTMLDivElement | null = $state(null);
  let adjustedX = $state(0);
  let adjustedY = $state(0);

  // Calculate adjusted position based on viewport
  $effect(() => {
    // Start with the original position
    adjustedX = x;
    adjustedY = y;

    // Use requestAnimationFrame to get accurate measurements after render
    requestAnimationFrame(() => {
      if (menuRef) {
        const rect = menuRef.getBoundingClientRect();
        const viewportWidth = window.innerWidth;
        const viewportHeight = window.innerHeight;

        // Adjust X if menu would overflow right edge
        if (x + rect.width > viewportWidth) {
          adjustedX = viewportWidth - rect.width - 8;
        }

        // Adjust Y if menu would overflow bottom edge
        if (y + rect.height > viewportHeight) {
          adjustedY = viewportHeight - rect.height - 8;
        }
      }
    });
  });

  function handleClickOutside(event: MouseEvent) {
    if (menuRef && !menuRef.contains(event.target as Node)) {
      onClose();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    }
  }

  function handleItemClick(item: MenuItem) {
    if (!item.disabled) {
      item.action();
      onClose();
    }
  }
</script>

<svelte:window on:click={handleClickOutside} on:keydown={handleKeydown} />

<div
  bind:this={menuRef}
  class="fixed z-50 min-w-[160px] bg-bg-panel border border-bg-card rounded-lg shadow-lg py-1 overflow-hidden"
  style="left: {adjustedX}px; top: {adjustedY}px;"
  role="menu"
  tabindex="-1"
>
  {#each items as item, index (index)}
    {#if item.divider}
      <div class="h-px bg-bg-card my-1"></div>
    {:else}
      <button
        type="button"
        role="menuitem"
        class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left transition-colors"
        class:text-text-primary={!item.disabled && !item.danger}
        class:text-red-400={item.danger && !item.disabled}
        class:text-text-secondary={item.disabled}
        class:opacity-50={item.disabled}
        class:cursor-not-allowed={item.disabled}
        class:hover:bg-bg-card={!item.disabled}
        disabled={item.disabled}
        onclick={() => handleItemClick(item)}
      >
        {#if item.icon}
          {@const Icon = item.icon}
          <Icon class="w-4 h-4" />
        {/if}
        <span>{item.label}</span>
      </button>
    {/if}
  {/each}
</div>
