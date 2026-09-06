<script lang="ts">
  import { ExternalLink } from "lucide-svelte";
  import type { FieldDefinition } from "../types";

  let {
    definition,
    value = null,
    disabled = false,
    onChange,
  }: {
    definition: FieldDefinition;
    value: string | null;
    disabled?: boolean;
    onChange: (value: string | null) => void;
  } = $props();

  let selectOptions = $derived.by<string[]>(() => {
    if (!definition.options) return [];
    try {
      return JSON.parse(definition.options) as string[];
    } catch {
      return [];
    }
  });

  function handleTextInput(e: Event) {
    const target = e.target as HTMLInputElement;
    onChange(target.value || null);
  }

  function handleNumberInput(e: Event) {
    const target = e.target as HTMLInputElement;
    onChange(target.value || null);
  }

  function handleDateInput(e: Event) {
    const target = e.target as HTMLInputElement;
    onChange(target.value || null);
  }

  function handleSelectInput(e: Event) {
    const target = e.target as HTMLSelectElement;
    onChange(target.value || null);
  }

  function handleCheckboxInput(e: Event) {
    const target = e.target as HTMLInputElement;
    onChange(target.checked ? "true" : "false");
  }

  function handleMultiselectToggle(option: string) {
    let current: string[] = [];
    if (value) {
      try {
        current = JSON.parse(value) as string[];
      } catch {
        current = [];
      }
    }

    if (current.includes(option)) {
      current = current.filter((v) => v !== option);
    } else {
      current = [...current, option];
    }

    onChange(current.length > 0 ? JSON.stringify(current) : null);
  }

  function isMultiselectSelected(option: string): boolean {
    if (!value) return false;
    try {
      const arr = JSON.parse(value) as string[];
      return arr.includes(option);
    } catch {
      return false;
    }
  }

  const inputClass =
    "w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2 text-press-ui focus:outline-none focus:border-press-accent";
</script>

<div>
  <label class="block text-press-ui text-press-muted mb-1">
    {definition.name}
    {#if definition.required}<span class="text-press-error">*</span>{/if}
  </label>

  {#if definition.field_type === "text"}
    <input
      type="text"
      value={value ?? ""}
      oninput={handleTextInput}
      class={inputClass}
      placeholder={definition.default_value ?? ""}
      {disabled}
    />
  {:else if definition.field_type === "number"}
    <input
      type="number"
      value={value ?? ""}
      oninput={handleNumberInput}
      class={inputClass}
      placeholder={definition.default_value ?? ""}
      {disabled}
    />
  {:else if definition.field_type === "date"}
    <input
      type="date"
      value={value ?? ""}
      oninput={handleDateInput}
      class={inputClass}
      {disabled}
    />
  {:else if definition.field_type === "url"}
    <div class="flex gap-2 items-center">
      <input
        type="url"
        value={value ?? ""}
        oninput={handleTextInput}
        class="{inputClass} flex-1"
        placeholder={definition.default_value ?? "https://..."}
        {disabled}
      />
      {#if value}
        <a
          href={value}
          target="_blank"
          rel="noopener noreferrer"
          class="text-press-accent-text hover:text-press-accent-text p-2"
          aria-label="Open URL"
        >
          <ExternalLink class="w-4 h-4" />
        </a>
      {/if}
    </div>
  {:else if definition.field_type === "select"}
    <select value={value ?? ""} onchange={handleSelectInput} class={inputClass} {disabled}>
      <option value="">— Select —</option>
      {#each selectOptions as option}
        <option value={option}>{option}</option>
      {/each}
    </select>
  {:else if definition.field_type === "multiselect" || definition.field_type === "multi_select"}
    <div class="flex flex-wrap gap-2">
      {#each selectOptions as option}
        <label
          class="inline-flex items-center gap-1.5 text-press-ui text-press-text cursor-pointer"
        >
          <input
            type="checkbox"
            class="accent-accent"
            checked={isMultiselectSelected(option)}
            onchange={() => handleMultiselectToggle(option)}
            {disabled}
          />
          {option}
        </label>
      {/each}
    </div>
  {:else if definition.field_type === "checkbox"}
    <label class="inline-flex items-center gap-2 text-press-ui text-press-text cursor-pointer">
      <input
        type="checkbox"
        class="accent-accent"
        checked={value === "true"}
        onchange={handleCheckboxInput}
        {disabled}
      />
      {definition.name}
    </label>
  {:else}
    <input
      type="text"
      value={value ?? ""}
      oninput={handleTextInput}
      class={inputClass}
      {disabled}
    />
  {/if}
</div>
