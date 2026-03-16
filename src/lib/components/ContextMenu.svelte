<script lang="ts">
  /* eslint-disable no-undef */
  import { ChevronRight } from "lucide-svelte";

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  type IconComponent = any;

  export interface MenuItem {
    label: string;
    icon?: IconComponent;
    action: () => void | Promise<void>;
    disabled?: boolean;
    danger?: boolean;
    divider?: boolean;
    children?: MenuItem[];
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
  let openSubmenuIndex: number | null = $state(null);
  let submenuRef: HTMLDivElement | null = $state(null);
  let hoverTimeout: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    adjustedX = x;
    adjustedY = y;

    requestAnimationFrame(() => {
      if (menuRef) {
        const rect = menuRef.getBoundingClientRect();
        const viewportWidth = window.innerWidth;
        const viewportHeight = window.innerHeight;

        if (x + rect.width > viewportWidth) {
          adjustedX = viewportWidth - rect.width - 8;
        }

        if (y + rect.height > viewportHeight) {
          adjustedY = viewportHeight - rect.height - 8;
        }
      }
    });
  });

  function handleClickOutside(event: MouseEvent) {
    if (
      menuRef &&
      !menuRef.contains(event.target as Node) &&
      (!submenuRef || !submenuRef.contains(event.target as Node))
    ) {
      onClose();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    }
  }

  function handleItemClick(item: MenuItem) {
    if (!item.disabled && !item.children) {
      item.action();
      onClose();
    }
  }

  function handleSubmenuItemClick(child: MenuItem) {
    if (!child.disabled) {
      child.action();
      onClose();
    }
  }

  function onItemMouseEnter(index: number, item: MenuItem) {
    if (hoverTimeout) clearTimeout(hoverTimeout);
    if (item.children && !item.disabled) {
      openSubmenuIndex = index;
    } else {
      hoverTimeout = setTimeout(() => {
        openSubmenuIndex = null;
      }, 150);
    }
  }

  function onSubmenuMouseEnter() {
    if (hoverTimeout) clearTimeout(hoverTimeout);
  }

  function onSubmenuMouseLeave() {
    hoverTimeout = setTimeout(() => {
      openSubmenuIndex = null;
    }, 150);
  }

  function getSubmenuPosition(index: number): { left: string; top: string } {
    if (!menuRef) return { left: "0px", top: "0px" };
    const menuRect = menuRef.getBoundingClientRect();
    const itemElements = menuRef.querySelectorAll("[role='menuitem'], [data-divider]");
    const itemEl = itemElements[index] as HTMLElement | undefined;
    if (!itemEl) return { left: `${menuRect.right}px`, top: `${menuRect.top}px` };

    const itemRect = itemEl.getBoundingClientRect();
    const viewportWidth = window.innerWidth;

    const fitsRight = menuRect.right + 160 < viewportWidth;
    const left = fitsRight ? `${menuRect.right - 4}px` : `${menuRect.left - 160 + 4}px`;
    const top = `${itemRect.top - 4}px`;

    return { left, top };
  }
</script>

<svelte:window on:click={handleClickOutside} on:keydown={handleKeydown} />

<div
  bind:this={menuRef}
  data-testid="context-menu"
  class="fixed z-50 min-w-[160px] bg-bg-panel border border-bg-card rounded-lg shadow-lg py-1 overflow-hidden"
  style="left: {adjustedX}px; top: {adjustedY}px;"
  role="menu"
  tabindex="-1"
>
  {#each items as item, index (index)}
    {#if item.divider}
      <div data-divider class="h-px bg-bg-card my-1"></div>
    {:else}
      <button
        type="button"
        role="menuitem"
        data-testid="context-menu-item"
        data-label={item.label}
        class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left transition-colors cursor-default"
        class:text-text-primary={!item.disabled && !item.danger}
        class:text-red-400={item.danger && !item.disabled}
        class:text-text-secondary={item.disabled}
        class:opacity-50={item.disabled}
        class:cursor-not-allowed={item.disabled}
        class:hover:bg-bg-card={!item.disabled}
        disabled={item.disabled && !item.children}
        onmouseenter={() => onItemMouseEnter(index, item)}
        onclick={() => handleItemClick(item)}
      >
        {#if item.icon}
          {@const Icon = item.icon}
          <Icon class="w-4 h-4" />
        {/if}
        <span class="flex-1">{item.label}</span>
        {#if item.children}
          <ChevronRight class="w-3.5 h-3.5 text-text-secondary" />
        {/if}
      </button>

      {#if item.children && openSubmenuIndex === index}
        {@const pos = getSubmenuPosition(index)}
        <div
          bind:this={submenuRef}
          class="fixed z-[60] min-w-[160px] bg-bg-panel border border-bg-card rounded-lg shadow-lg py-1 overflow-hidden"
          style="left: {pos.left}; top: {pos.top};"
          role="menu"
          tabindex="-1"
          onmouseenter={onSubmenuMouseEnter}
          onmouseleave={onSubmenuMouseLeave}
        >
          {#each item.children as child, ci (ci)}
            {#if child.divider}
              <div class="h-px bg-bg-card my-1"></div>
            {:else}
              <button
                type="button"
                role="menuitem"
                class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left transition-colors"
                class:text-text-primary={!child.disabled && !child.danger}
                class:text-red-400={child.danger && !child.disabled}
                class:text-text-secondary={child.disabled}
                class:opacity-50={child.disabled}
                class:cursor-not-allowed={child.disabled}
                class:hover:bg-bg-card={!child.disabled}
                disabled={child.disabled}
                onclick={() => handleSubmenuItemClick(child)}
              >
                {#if child.icon}
                  {@const ChildIcon = child.icon}
                  <ChildIcon class="w-4 h-4" />
                {/if}
                <span>{child.label}</span>
              </button>
            {/if}
          {/each}
        </div>
      {/if}
    {/if}
  {/each}
</div>
