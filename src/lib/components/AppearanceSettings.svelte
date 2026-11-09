<script lang="ts">
  import { Select } from "$press/svelte";
  import { ui } from "../stores/ui.svelte";
  import { LANGUAGE_NAMES, LOCALES, isLocale, t } from "../i18n";
  import type { Locale } from "../i18n/types";
  import { getLanguagePreference, setLanguagePreference } from "../utils/language";

  let { onStartTour }: { onStartTour?: () => void } = $props();

  // Each tip shows once; this brings them back without switching tips off and on.
  let tipsReset = $state(false);

  // Languages are listed by their own names, so a writer finds theirs whatever the
  // interface reads; only "System default" is translated.
  let language = $state<string>(getLanguagePreference());
  const languageOptions = $derived([
    { value: "system", label: t("settings.appearance.languageSystem") },
    ...(Object.keys(LOCALES) as Locale[])
      .sort((a, b) => (a === "en" ? -1 : b === "en" ? 1 : 0))
      .map((locale) => ({ value: locale, label: LANGUAGE_NAMES[locale] })),
  ]);
  function chooseLanguage(value: string) {
    void setLanguagePreference(isLocale(value) ? value : "system");
  }
  function showTipsAgain() {
    ui.resetGuidanceTooltips();
    tipsReset = true;
  }
</script>

<div class="settings-pane">
  <p class="settings-lede">
    {t("settings.appearance.lede")}
  </p>
  <div class="ka-group">
    <fieldset class="settings-fieldset">
      <legend class="ka-group-title">{t("settings.appearance.title")}</legend>
      <div class="ka-segments">
        <div
          class="ka-segment-track"
          role="radiogroup"
          aria-label={t("settings.appearance.themeAria")}
        >
          {#each [{ value: "dark", label: t("settings.appearance.themeDark") }, { value: "light", label: t("settings.appearance.themeLight") }, { value: "system", label: t("settings.appearance.themeSystem") }] as opt}
            <label class="ka-segment" class:ka-selected={ui.theme === opt.value}>
              <input
                type="radio"
                name="theme"
                data-testid="theme-option-{opt.value}"
                value={opt.value}
                checked={ui.theme === opt.value}
                onchange={() => ui.setTheme(opt.value as "dark" | "light" | "system")}
              />
              <span>{opt.label}</span>
            </label>
          {/each}
        </div>
      </div>
      <p class="ka-help">{t("settings.appearance.systemHelp")}</p>
    </fieldset>
  </div>

  <div class="ka-group" data-testid="language-setting">
    <Select
      label={t("settings.appearance.language")}
      options={languageOptions}
      helper={t("settings.appearance.languageHelp")}
      bind:value={language}
      onChange={chooseLanguage}
    />
  </div>

  <div class="ka-group">
    <fieldset class="settings-fieldset">
      <legend class="ka-group-title">{t("settings.appearance.guidance")}</legend>
      <div class="ka-check">
        <input
          id="guidance-enabled"
          type="checkbox"
          checked={ui.guidanceEnabled}
          onchange={(e) => ui.setGuidanceEnabled((e.target as HTMLInputElement).checked)}
          aria-describedby="guidance-enabled-help"
        />
        <label for="guidance-enabled">{t("settings.appearance.showTips")}</label>
      </div>
      <p id="guidance-enabled-help" class="ka-help">
        {t("settings.appearance.tipsHelp")}
      </p>
      <div class="guidance-actions">
        <button
          type="button"
          class="ka-button ka-button--secondary"
          onclick={showTipsAgain}
          disabled={!ui.guidanceEnabled}
          aria-describedby="guidance-reset-status"
        >
          {t("settings.appearance.showTipsAgain")}
        </button>
        {#if onStartTour}
          <button type="button" class="ka-button ka-button--secondary" onclick={onStartTour}>
            {t("settings.appearance.takeTour")}
          </button>
        {/if}
      </div>
      <p id="guidance-reset-status" class="ka-help" role="status">
        {#if !ui.guidanceEnabled}
          {t("settings.appearance.turnOnTips")}
        {:else if tipsReset}
          {t("settings.appearance.tipsReset")}
        {/if}
      </p>
    </fieldset>
  </div>
</div>

<style>
  .guidance-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2xs);
  }
  #guidance-reset-status:empty {
    display: none;
  }
</style>
