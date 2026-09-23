<script lang="ts">
let {label,value,max=100}:{label:string;value:number;max?:number}=$props();
const limit=$derived(Math.max(1,max));
</script>
<div class="ka-progress od-field"><div class="ka-between"><span>{label}</span><span class="od-nowrap">{Math.max(0,value).toLocaleString()} / {limit.toLocaleString()}</span></div><progress aria-label={label} value={Math.max(0,Math.min(value,limit))} max={limit}></progress></div>
