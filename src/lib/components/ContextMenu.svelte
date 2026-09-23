<script module lang="ts">
  import type { ComponentType } from "svelte";

  export interface MenuItem {
    label: string;
    icon?: ComponentType;
    action: () => void | Promise<void>;
    disabled?: boolean;
    danger?: boolean;
    divider?: boolean;
    children?: MenuItem[];
  }
</script>

<script lang="ts">
  import { ChevronRight } from "lucide-svelte";
  import { onDestroy, onMount, tick } from "svelte";

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
  // Whatever had focus when the menu opened gets it back when the menu closes.
  const returnFocus =
    typeof document !== "undefined" && document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;

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

  onMount(() => {
    menuItems(menuRef)[0]?.focus({ preventScroll: true });
  });

  onDestroy(() => {
    if (hoverTimeout) clearTimeout(hoverTimeout);
    // Only reclaim focus that would otherwise be lost; an owner that already
    // moved focus somewhere deliberate keeps it there.
    const active = document.activeElement;
    const lost =
      !active ||
      active === document.body ||
      !!menuRef?.contains(active) ||
      !!submenuRef?.contains(active);
    if (lost && returnFocus?.isConnected) returnFocus.focus({ preventScroll: true });
  });

  function menuItems(root: HTMLElement | null): HTMLButtonElement[] {
    return root
      ? [...root.querySelectorAll<HTMLButtonElement>(":scope > [role='menuitem']:not(:disabled)")]
      : [];
  }

  function handleClickOutside(event: MouseEvent) {
    if (
      menuRef &&
      !menuRef.contains(event.target as Node) &&
      (!submenuRef || !submenuRef.contains(event.target as Node))
    ) {
      onClose();
    }
  }

  async function openSubmenu(index: number) {
    openSubmenuIndex = index;
    await tick();
    menuItems(submenuRef)[0]?.focus({ preventScroll: true });
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      if (openSubmenuIndex !== null && submenuRef?.contains(document.activeElement)) {
        const parent = openSubmenuIndex;
        openSubmenuIndex = null;
        menuRef?.querySelectorAll<HTMLButtonElement>("[data-item-index]")[parent]?.focus();
        return;
      }
      onClose();
      return;
    }
    const inSubmenu = !!submenuRef?.contains(document.activeElement);
    const list = menuItems(inSubmenu ? submenuRef : menuRef);
    if (!list.length) return;
    const current = list.indexOf(document.activeElement as HTMLButtonElement);
    let next: number | null = null;
    if (event.key === "ArrowDown") next = (current + 1) % list.length;
    else if (event.key === "ArrowUp") next = (current - 1 + list.length) % list.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = list.length - 1;
    else if (event.key === "ArrowRight" && !inSubmenu) {
      const index = Number((document.activeElement as HTMLElement)?.dataset.itemIndex);
      if (items[index]?.children && !items[index].disabled) {
        event.preventDefault();
        void openSubmenu(index);
      }
      return;
    } else if (event.key === "ArrowLeft" && inSubmenu && openSubmenuIndex !== null) {
      event.preventDefault();
      const parent = openSubmenuIndex;
      openSubmenuIndex = null;
      menuRef?.querySelectorAll<HTMLButtonElement>("[data-item-index]")[parent]?.focus();
      return;
    }
    if (next === null) return;
    event.preventDefault();
    list[next]?.focus();
  }

  function handleItemClick(item: MenuItem, index: number) {
    if (item.disabled) return;
    if (item.children) {
      void openSubmenu(index);
      return;
    }
    item.action();
    onClose();
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
    const itemEl = menuRef.querySelectorAll<HTMLElement>("[data-item-index]")[index];
    if (!itemEl) return { left: `${menuRect.right}px`, top: `${menuRect.top}px` };

    const itemRect = itemEl.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const submenuWidth = 220;

    const fitsRight = menuRect.right + submenuWidth < viewportWidth;
    const left = fitsRight ? `${menuRect.right - 4}px` : `${menuRect.left - submenuWidth + 4}px`;
    const top = `${itemRect.top - 8}px`;

    return { left, top };
  }
</script>

<svelte:window onclick={handleClickOutside} onkeydown={handleKeydown} />

<div
  bind:this={menuRef}
  data-testid="context-menu"
  class="ka-menu-list context-menu"
  style="left: {adjustedX}px; top: {adjustedY}px;"
  role="menu"
  tabindex="-1"
>
  {#each items as item, index (index)}
    {#if item.divider}
      <div data-divider role="separator"></div>
    {:else}
      <button
        type="button"
        role="menuitem"
        data-testid="context-menu-item"
        data-label={item.label}
        data-item-index={index}
        class:ka-menu-danger={item.danger}
        aria-haspopup={item.children ? "menu" : undefined}
        aria-expanded={item.children ? openSubmenuIndex === index : undefined}
        disabled={item.disabled && !item.children}
        aria-disabled={item.disabled || undefined}
        onmouseenter={() => onItemMouseEnter(index, item)}
        onclick={() => handleItemClick(item, index)}
      >
        <span>
          {#if item.icon}
            {@const Icon = item.icon}
            <Icon class="w-5 h-5 ka-icon" aria-hidden="true" />
          {:else}
            <span class="ka-icon-slot" aria-hidden="true"></span>
          {/if}
          {item.label}
        </span>
        {#if item.children}
          <ChevronRight class="w-5 h-5 ka-icon" aria-hidden="true" />
        {/if}
      </button>
    {/if}
  {/each}
</div>

{#each items as item, index (index)}
  {#if item.children && openSubmenuIndex === index}
    {@const pos = getSubmenuPosition(index)}
    <div
      bind:this={submenuRef}
      class="ka-menu-list context-menu"
      style="left: {pos.left}; top: {pos.top};"
      role="menu"
      aria-label={item.label}
      tabindex="-1"
      onmouseenter={onSubmenuMouseEnter}
      onmouseleave={onSubmenuMouseLeave}
    >
      {#each item.children as child, ci (ci)}
        {#if child.divider}
          <div role="separator"></div>
        {:else}
          <button
            type="button"
            role="menuitem"
            class:ka-menu-danger={child.danger}
            disabled={child.disabled}
            onclick={() => handleSubmenuItemClick(child)}
          >
            <span>
              {#if child.icon}
                {@const ChildIcon = child.icon}
                <ChildIcon class="w-5 h-5 ka-icon" aria-hidden="true" />
              {:else}
                <span class="ka-icon-slot" aria-hidden="true"></span>
              {/if}
              {child.label}
            </span>
          </button>
        {/if}
      {/each}
    </div>
  {/if}
{/each}

<style>
  /* Press menu: 44px items, 20px icons, hairline separators, overlay shadow. */
  .context-menu {
    position: fixed;
    z-index: var(--z-popover);
    min-width: 200px;
    max-width: 320px;
    margin: 0;
    box-shadow: var(--shadow-overlay);
  }
  .context-menu:focus {
    outline: none;
  }
  .context-menu > button:focus-visible {
    outline: 2px solid var(--color-accent-text);
    outline-offset: -2px;
  }
  .context-menu > button[aria-disabled="true"] {
    color: var(--color-disabled-text);
    cursor: not-allowed;
  }
</style>
