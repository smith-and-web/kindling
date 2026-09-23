<script lang="ts">
let { label, value = $bindable(''), type = 'text', multiline = false, helper = '', error = '', required = false, disabled = false, readonly = false, placeholder = '', name, id, onChange, onBlur }: {
label:string; value?:string; type?:'text'|'email'|'number'|'date'|'url'|'search'; multiline?:boolean; helper?:string; error?:string; required?:boolean; disabled?:boolean; readonly?:boolean; placeholder?:string; name?:string; id?:string; onChange?:(value:string)=>void; onBlur?:(value:string)=>void;
} = $props();
const uid=$props.id();
let localError=$state('');
const fieldId=$derived(id || uid);
const message=$derived(error || localError);
function change(e:Event) { value=(e.currentTarget as HTMLInputElement).value; localError=''; onChange?.(value); }
function blur(e:FocusEvent) { const el=e.currentTarget as HTMLInputElement; localError=el.validity.valid ? '' : el.validity.valueMissing ? `Enter ${label.toLowerCase()} to continue.` : `Enter a valid ${type === 'text' ? 'value' : type}.`; onBlur?.(value); }
</script>
<div class="ka-field od-field">
<label for={fieldId}>{label}{#if required}<span aria-hidden="true"> *</span><span class="ka-sr"> (required)</span>{/if}</label>
{#if multiline}<textarea id={fieldId} {name} {value} {required} {disabled} {readonly} {placeholder} rows="4" aria-invalid={!!message} aria-describedby={`${fieldId}-help ${fieldId}-error`} oninput={change} onblur={blur}></textarea>
{:else}<input id={fieldId} {name} {type} {value} {required} {disabled} {readonly} {placeholder} aria-invalid={!!message} aria-describedby={`${fieldId}-help ${fieldId}-error`} oninput={change} onblur={blur}/>{/if}
<p id={`${fieldId}-help`} class="ka-help" hidden={!helper}>{helper}</p><p id={`${fieldId}-error`} class="ka-error" hidden={!message}>{message}</p>
</div>
