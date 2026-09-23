<script lang="ts">
import type {Snippet} from 'svelte'; import Icon from './Icon.svelte';
let {title,message,tone='info',children}:{title:string;message:string;tone?:'info'|'success'|'error'|'warning';children?:Snippet}=$props();
</script>
<div class={`ka-notice ka-notice--${tone} od-row-top`} role={tone==='error'?'alert':'status'}><Icon name={tone==='success'?'check':tone==='error'?'triangle-alert':'info'}/><div class="od-field od-fill"><strong>{title}</strong><p>{message}</p>{@render children?.()}</div></div>
