<script lang="ts">
import { icons } from './icons';
let { name, size = 20 }: { name: string; size?: number } = $props();
</script>
<svg class="ka-icon" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">
{#each icons[name] || icons['info'] as [tag, attrs]}<svelte:element this={tag} {...attrs}/>{/each}
</svg>
