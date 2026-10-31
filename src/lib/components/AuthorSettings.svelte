<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Loader2 } from "lucide-svelte";
  import type { AppSettings } from "../types";
  import { t } from "../i18n";

  let { dirty = $bindable(false), busy = $bindable(false) }: { dirty?: boolean; busy?: boolean } =
    $props();
  let baseline = $state("");
  let loaded = $state(false);
  let saved = $state(false);
  // Form state
  let authorName = $state("");
  let addressLine1 = $state("");
  let addressLine2 = $state("");
  let phone = $state("");
  let email = $state("");

  let loading = $state(true);
  let saving = $state(false);
  let error = $state<string | null>(null);

  const snapshot = $derived(JSON.stringify([authorName, addressLine1, addressLine2, phone, email]));
  $effect(() => {
    dirty = loaded && snapshot !== baseline;
    busy = saving;
  });
  // Load existing settings on mount
  $effect(() => {
    loadSettings();
  });

  async function loadSettings() {
    loading = true;
    error = null;

    try {
      const settings = await invoke<AppSettings>("get_app_settings");
      authorName = settings.author_name ?? "";
      addressLine1 = settings.contact_address_line1 ?? "";
      addressLine2 = settings.contact_address_line2 ?? "";
      phone = settings.contact_phone ?? "";
      email = settings.contact_email ?? "";
      baseline = JSON.stringify([authorName, addressLine1, addressLine2, phone, email]);
      loaded = true;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function resetSettings() {
    if (saving) return;
    saving = true;
    error = null;
    try {
      await invoke("reset_app_settings");
      await loadSettings();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  async function handleSave() {
    if (saving || !loaded) return;
    saving = true;
    saved = false;
    error = null;

    try {
      // Convert empty strings to null for optional fields
      const settings: AppSettings = {
        author_name: authorName.trim() || null,
        contact_address_line1: addressLine1.trim() || null,
        contact_address_line2: addressLine2.trim() || null,
        contact_phone: phone.trim() || null,
        contact_email: email.trim() || null,
      };

      await invoke<AppSettings>("update_app_settings", {
        settings,
      });

      baseline = snapshot;
      saved = true;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }
</script>

<div class="settings-pane" aria-busy={busy}>
  <p class="settings-lede">
    {t("settings.author.lede")}
  </p>
  {#if loading}<p role="status" class="ka-help">{t("settings.author.loading")}</p>
  {:else if loaded}
    <div class="ka-group">
      <fieldset class="settings-fieldset">
        <legend class="ka-group-title">{t("settings.author.infoTitle")}</legend>
        <div class="ka-field od-field">
          <label for="author-name">{t("settings.author.nameLabel")}</label>
          <input
            id="author-name"
            type="text"
            bind:value={authorName}
            placeholder={t("settings.author.namePlaceholder")}
            disabled={saving}
            aria-describedby="author-name-help"
          />
          <p id="author-name-help" class="ka-help">
            {t("settings.author.nameHelp")}
          </p>
        </div>
      </fieldset>
    </div>

    <div class="ka-group">
      <fieldset class="settings-fieldset">
        <legend class="ka-group-title">{t("settings.author.contactTitle")}</legend>
        <p class="ka-help">
          {t("settings.author.contactHelp")}
        </p>
        <div class="ka-field od-field">
          <label for="address-line1">{t("settings.author.address1")}</label>
          <input
            id="address-line1"
            type="text"
            bind:value={addressLine1}
            placeholder={t("settings.author.address1Placeholder")}
            disabled={saving}
          />
        </div>
        <div class="ka-field od-field">
          <label for="address-line2">{t("settings.author.address2")}</label>
          <input
            id="address-line2"
            type="text"
            bind:value={addressLine2}
            placeholder={t("settings.author.address2Placeholder")}
            disabled={saving}
          />
        </div>
        <div class="settings-pair">
          <div class="ka-field od-field">
            <label for="phone">{t("settings.author.phone")}</label>
            <input
              id="phone"
              type="tel"
              bind:value={phone}
              placeholder={t("settings.author.phonePlaceholder")}
              disabled={saving}
            />
          </div>
          <div class="ka-field od-field">
            <label for="email">{t("settings.author.email")}</label>
            <input
              id="email"
              type="email"
              bind:value={email}
              placeholder={t("settings.author.emailPlaceholder")}
              disabled={saving}
            />
          </div>
        </div>
      </fieldset>
    </div>

    <div class="settings-actions">
      {#if saved && !dirty}<p role="status" class="ka-help">{t("settings.author.saved")}</p>{/if}
      <button
        type="button"
        onclick={handleSave}
        disabled={saving}
        aria-busy={saving || undefined}
        class="ka-button"
        >{#if saving}<Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
          {t("common.saving")}{:else}{t("settings.author.save")}{/if}</button
      >
    </div>
  {:else}
    <div class="ka-notice ka-notice--warning od-row-top author-recovery">
      <div class="od-field od-fill">
        <strong>{t("settings.author.loadFailedTitle")}</strong>
        <p>
          {t("settings.author.loadFailedHelp")}
        </p>
      </div>
    </div>
    <div class="settings-actions">
      <button
        type="button"
        class="ka-button ka-button--secondary"
        onclick={loadSettings}
        disabled={saving}>{t("settings.author.retry")}</button
      >
      <button
        type="button"
        class="ka-button ka-button--danger"
        onclick={resetSettings}
        disabled={saving}
        aria-busy={saving || undefined}>{t("settings.author.reset")}</button
      >
    </div>
  {/if}
  {#if error}<p role="alert" class="ka-error">{error}</p>{/if}
</div>
