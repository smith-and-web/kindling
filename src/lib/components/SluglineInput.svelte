<script lang="ts">
  /**
   * SluglineInput - Scene title input for screenplay projects
   *
   * Format: INT. LOCATION - DAY (or EXT., various times)
   * Provides INT/EXT prefix, location suggestions from project, time-of-day options.
   */
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
  let isSluglineFormat = $state(false);

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
    const upper = value.trim().toUpperCase();
    isSluglineFormat = upper.startsWith("INT.") || upper.startsWith("EXT.");
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
    const result = isSluglineFormat ? buildSlugline().trim() : location.trim();
    if (result) onSave(result);
  }

  function setPrefix(p: "INT" | "EXT") {
    prefix = p;
    isSluglineFormat = true;
    emitSave();
  }

  function setTime(t: string) {
    timeOfDay = t;
    isSluglineFormat = true;
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

<div class="slugline {className}" role="group" aria-label="Scene heading">
  <div class="ka-segment-track slugline-prefix" role="group" aria-label="Interior or exterior">
    {#each ["INT", "EXT"] as const as option (option)}
      <button
        type="button"
        class="ka-segment"
        class:ka-selected={prefix === option}
        aria-pressed={prefix === option}
        onclick={() => setPrefix(option)}
        {disabled}
      >
        {option}.
      </button>
    {/each}
  </div>

  <input
    bind:value={location}
    oninput={handleLocationInput}
    onblur={handleLocationBlur}
    type="text"
    list="slugline-locations"
    placeholder="Location"
    aria-label="Location"
    {disabled}
    class="slugline-location"
  />
  <datalist id="slugline-locations">
    {#each locationSuggestions as loc}
      <option value={loc}></option>
    {/each}
  </datalist>

  <select
    class="slugline-time"
    aria-label="Time of day"
    value={timeOfDay}
    onchange={(event) => setTime((event.currentTarget as HTMLSelectElement).value)}
    {disabled}
  >
    {#if !TIME_OPTIONS.includes(timeOfDay as (typeof TIME_OPTIONS)[number])}
      <option value={timeOfDay}>{timeOfDay}</option>
    {/if}
    {#each TIME_OPTIONS as t (t)}
      <option value={t}>{t}</option>
    {/each}
  </select>
</div>

<style>
  .slugline {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-xs);
  }
  .slugline-prefix {
    flex: none;
  }
  .slugline-prefix .ka-segment {
    min-width: 64px;
    letter-spacing: 0.02em;
  }
  .slugline-location,
  .slugline-time {
    min-height: var(--control-target);
    padding: 9px var(--space-xs);
    border: 1px solid var(--color-control-border-hover);
    border-radius: var(--radius-m);
    background-color: var(--color-surface-sunken);
    color: var(--color-text);
    font: var(--text-base) / 1.5 var(--font-ui);
    letter-spacing: 0.02em;
  }
  .slugline-location {
    flex: 1 1 200px;
    min-width: 0;
    text-transform: uppercase;
  }
  .slugline-location::placeholder {
    text-transform: none;
    letter-spacing: 0;
    color: var(--color-text-muted);
  }
  .slugline-time {
    flex: none;
    padding-right: 40px;
  }
</style>
