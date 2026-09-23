<script lang="ts">
import type { Snippet } from 'svelte';
import type { HTMLButtonAttributes } from 'svelte/elements';
let { children, variant = 'primary', busy = false, disabled = false, type = 'button', class: extra = '', ...rest }: HTMLButtonAttributes & { children?: Snippet; variant?: 'primary'|'secondary'|'ghost'|'danger'; busy?: boolean } = $props();
</script>
<button {...rest} {type} disabled={disabled || busy} aria-busy={busy || undefined} class={`ka-button ka-button--${variant} ${extra}`}>{@render children?.()}</button>
