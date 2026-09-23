<script lang="ts">
let {label,tone='neutral'}:{label:string;tone?:'neutral'|'accent'|'success'|'error'|'warning'}=$props();
</script>
<span class={`ka-badge ka-badge--${tone}`}>{label}</span>
