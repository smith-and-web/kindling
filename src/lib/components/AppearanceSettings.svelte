<script lang="ts">
  import { Palette, Lightbulb } from "lucide-svelte";
  import { ui } from "../stores/ui.svelte";
</script>

<div class="space-y-6">
  <p class="text-press-ui text-press-muted">
    Choose how Kindling looks and how much guidance it shows. Changes apply immediately to all
    projects.
  </p>
  <!-- Section: Appearance -->
  <fieldset>
    <legend class="flex items-center gap-2 text-press-ui font-medium text-press-accent-text mb-3">
      <Palette class="w-4 h-4" />
      Appearance
    </legend>
    <div class="flex gap-3">
      {#each [{ value: "dark", label: "Dark" }, { value: "light", label: "Light" }, { value: "system", label: "System" }] as opt}
        <label
          class="flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded-lg border cursor-pointer transition-colors focus-within:ring-2 focus-within:ring-press-focus {ui.theme ===
          opt.value
            ? 'border-press-accent bg-press-accent-wash text-press-text'
            : 'border-press-border bg-press-sunken text-press-muted hover:text-press-text'}"
        >
          <input
            type="radio"
            name="theme"
            data-testid="theme-option-{opt.value}"
            value={opt.value}
            checked={ui.theme === opt.value}
            onchange={() => ui.setTheme(opt.value as "dark" | "light" | "system")}
            class="sr-only"
          />
          <span class="text-press-ui">{opt.label}</span>
        </label>
      {/each}
    </div>
    <p class="text-press-eyebrow text-press-muted mt-2">
      "System" follows your operating system's appearance setting.
    </p>
  </fieldset>

  <!-- Section: Guidance -->
  <fieldset>
    <legend class="flex items-center gap-2 text-press-ui font-medium text-press-accent-text mb-3">
      <Lightbulb class="w-4 h-4" />
      Guidance
    </legend>
    <label class="flex items-center gap-2 cursor-pointer">
      <input
        type="checkbox"
        checked={ui.guidanceEnabled}
        onchange={(e) => ui.setGuidanceEnabled((e.target as HTMLInputElement).checked)}
        class="rounded border-press-border text-press-accent-text focus:ring-press-focus"
      />
      <span class="text-press-ui text-press-text">Show guidance tips</span>
    </label>
    <p class="text-press-eyebrow text-press-muted mt-1 ml-6">
      Contextual tips on first visit to sidebar, scene panel, and references. Can be disabled for
      experienced users.
    </p>
  </fieldset>
</div>
