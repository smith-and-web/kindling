<script lang="ts">
import type { HTMLButtonAttributes } from 'svelte/elements';
import Icon from './Icon.svelte';
let { label, icon, class: extra = '', ...rest }: Omit<HTMLButtonAttributes,'children'> & {label:string;icon:string} = $props();
</script>
<button type="button" {...rest} aria-label={label} title={label} class={`ka-button ka-button--ghost ka-icon-button ${extra}`}><Icon name={icon}/></button>
