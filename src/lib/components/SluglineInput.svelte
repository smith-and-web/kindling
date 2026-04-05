<script lang="ts">
  /**
   * SluglineInput - Scene title input for screenplay projects
   *
   * Format: INT. LOCATION - DAY (or EXT., various times)
   * Provides INT/EXT prefix, location suggestions from project, time-of-day options.
   */
  import { ChevronDown } from "lucide-svelte";
  import type { Location } from "../types";

  const TIME_OPTIONS = [
    "DAY",
    "NIGHT",
    "DAWN",
    "DUSK",
    "SUNRISE",
    "SUNSET",
    "MORNING",
    "AFTERNOON",
    "EVENING",
    "LATER",
    "MOMENTS LATER",
    "CONTINUOUS",
    "SAME",
  ] as const;

  let {
    value,
    onSave,
    locations = [],
    disabled = false,
    class: className = "",
  }: {
    value: string;
    onSave: (slugline: string) => void | Promise<void>;
    locations?: Location[];
    disabled?: boolean;
    class?: string;
  } = $props();

  let prefix = $state<"INT" | "EXT">("INT");
  let location = $state("");
  let timeOfDay = $state("DAY");
  let showTimeDropdown = $state(false);

  // Parse existing slugline on init/change
  function parseSlugline(s: string) {
    const trimmed = s.trim();
    if (!trimmed) {
      return { prefix: "INT" as const, location: "", timeOfDay: "DAY" };
    }
    const upper = trimmed.toUpperCase();
    const hasIntPrefix = upper.startsWith("INT.");
    const hasExtPrefix = upper.startsWith("EXT.");
    if (!hasIntPrefix && !hasExtPrefix) {
      return { prefix: "INT" as const, location: trimmed, timeOfDay: "DAY" };
    }
    const prefixVal: "INT" | "EXT" = hasExtPrefix ? "EXT" : "INT";
    const rest = trimmed.slice(prefixVal.length + 1).trim();
    const dashIdx = rest.indexOf(" - ");
    let loc: string;
    let time = "DAY";
    if (dashIdx >= 0) {
      loc = rest.slice(0, dashIdx).trim();
      time =
        rest
          .slice(dashIdx + 3)
          .trim()
          .toUpperCase() || "DAY";
      if (!TIME_OPTIONS.includes(time as (typeof TIME_OPTIONS)[number])) {
        time = time || "DAY";
      }
    } else {
      loc = rest;
    }
    return { prefix: prefixVal, location: loc, timeOfDay: time || "DAY" };
  }

  $effect(() => {
    const parsed = parseSlugline(value);
    prefix = parsed.prefix;
    location = parsed.location;
    timeOfDay = parsed.timeOfDay;
  });

  function buildSlugline() {
    const loc = location.trim();
    const time = timeOfDay.trim();
    if (!loc) return `${prefix}.`;
    if (!time) return `${prefix}. ${loc}`;
    return `${prefix}. ${loc} - ${time}`;
  }

  function emitSave() {
    const slugline = buildSlugline().trim();
    if (slugline) onSave(slugline);
  }

  function setPrefix(p: "INT" | "EXT") {
    prefix = p;
    emitSave();
  }

  function setTime(t: string) {
    timeOfDay = t;
    showTimeDropdown = false;
    emitSave();
  }

  function handleLocationInput() {
    emitSave();
  }

  function handleLocationBlur() {
    emitSave();
  }

  const locationSuggestions = $derived(
    locations
      .map((l) => l.name)
      .filter((n) => n.toLowerCase().includes(location.toLowerCase().trim()))
      .slice(0, 8)
  );
</script>

<div class="flex flex-wrap items-center gap-2 {className}">
  <div class="flex rounded-lg border border-bg-card overflow-hidden">
    <button
      type="button"
      onclick={() => setPrefix("INT")}
      {disabled}
      class="px-3 py-2 text-sm font-medium transition-colors {prefix === 'INT'
        ? 'bg-accent text-white'
        : 'bg-bg-card text-text-secondary hover:text-text-primary'}"
    >
      INT.
    </button>
    <button
      type="button"
      onclick={() => setPrefix("EXT")}
      {disabled}
      class="px-3 py-2 text-sm font-medium transition-colors border-l border-bg-panel {prefix ===
      'EXT'
        ? 'bg-accent text-white'
        : 'bg-bg-card text-text-secondary hover:text-text-primary'}"
    >
      EXT.
    </button>
  </div>

  <input
    bind:value={location}
    oninput={handleLocationInput}
    onblur={handleLocationBlur}
    type="text"
    list="slugline-locations"
    placeholder="LOCATION"
    {disabled}
    class="flex-1 min-w-[120px] bg-bg-card text-text-primary text-sm border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent uppercase placeholder:normal-case placeholder:text-text-secondary/60"
  />
  <datalist id="slugline-locations">
    {#each locationSuggestions as loc}
      <option value={loc}></option>
    {/each}
  </datalist>

  <div class="relative">
    <button
      type="button"
      onclick={(e) => {
        e.stopPropagation();
        showTimeDropdown = !showTimeDropdown;
      }}
      {disabled}
      class="flex items-center gap-1.5 px-3 py-2 text-sm bg-bg-card text-text-primary border border-bg-card rounded-lg hover:border-accent/50 transition-colors disabled:opacity-60"
    >
      <span class="uppercase">{timeOfDay}</span>
      <ChevronDown
        class="w-4 h-4 text-text-secondary transition-transform {showTimeDropdown
          ? 'rotate-180'
          : ''}"
      />
    </button>
    {#if showTimeDropdown}
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
      <div
        class="absolute left-0 top-full mt-1 z-50 bg-bg-panel border border-bg-card rounded-lg shadow-lg py-1 max-h-48 overflow-y-auto"
        onclick={(e) => e.stopPropagation()}
      >
        {#each TIME_OPTIONS as t}
          <button
            type="button"
            onclick={() => setTime(t)}
            class="w-full text-left px-3 py-2 text-sm text-text-primary hover:bg-bg-card transition-colors {timeOfDay ===
            t
              ? 'bg-accent/20 text-accent'
              : ''}"
          >
            {t}
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<svelte:window onclick={() => (showTimeDropdown = false)} />
