<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Loader2 } from "lucide-svelte";
  import type { AppSettings } from "../types";

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
    Your author and contact details apply to manuscript title pages across all projects.
  </p>
  {#if loading}<p role="status" class="ka-help">Loading author details…</p>
  {:else if loaded}
    <div class="ka-group">
      <fieldset class="settings-fieldset">
        <legend class="ka-group-title">Author information</legend>
        <div class="ka-field od-field">
          <label for="author-name">Author Name</label>
          <input
            id="author-name"
            type="text"
            bind:value={authorName}
            placeholder="Your legal name"
            disabled={saving}
            aria-describedby="author-name-help"
          />
          <p id="author-name-help" class="ka-help">
            Used in contact info on title pages. Projects can override this with a pen name.
          </p>
        </div>
      </fieldset>
    </div>

    <div class="ka-group">
      <fieldset class="settings-fieldset">
        <legend class="ka-group-title">Contact information</legend>
        <p class="ka-help">
          Optional details for manuscript title pages. Use any format that works for your country.
        </p>
        <div class="ka-field od-field">
          <label for="address-line1">Address Line 1</label>
          <input
            id="address-line1"
            type="text"
            bind:value={addressLine1}
            placeholder="Street address or PO Box"
            disabled={saving}
          />
        </div>
        <div class="ka-field od-field">
          <label for="address-line2">Address Line 2</label>
          <input
            id="address-line2"
            type="text"
            bind:value={addressLine2}
            placeholder="City, State/Province, Postal Code, Country"
            disabled={saving}
          />
        </div>
        <div class="settings-pair">
          <div class="ka-field od-field">
            <label for="phone">Phone</label>
            <input
              id="phone"
              type="tel"
              bind:value={phone}
              placeholder="+1 (555) 123-4567"
              disabled={saving}
            />
          </div>
          <div class="ka-field od-field">
            <label for="email">Email</label>
            <input
              id="email"
              type="email"
              bind:value={email}
              placeholder="author@email.com"
              disabled={saving}
            />
          </div>
        </div>
      </fieldset>
    </div>

    <div class="settings-actions">
      {#if saved && !dirty}<p role="status" class="ka-help">Author details saved.</p>{/if}
      <button
        type="button"
        onclick={handleSave}
        disabled={saving}
        aria-busy={saving || undefined}
        class="ka-button"
        >{#if saving}<Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
          Saving…{:else}Save author details{/if}</button
      >
    </div>
  {:else}
    <div class="ka-notice ka-notice--warning od-row-top author-recovery">
      <div class="od-field od-fill">
        <strong>Author details couldn’t be loaded</strong>
        <p>
          If the settings file is damaged, reset author details to restore saving and exports. A
          recovery copy of the original file will be kept.
        </p>
      </div>
    </div>
    <div class="settings-actions">
      <button
        type="button"
        class="ka-button ka-button--secondary"
        onclick={loadSettings}
        disabled={saving}>Retry loading author details</button
      >
      <button
        type="button"
        class="ka-button ka-button--danger"
        onclick={resetSettings}
        disabled={saving}
        aria-busy={saving || undefined}>Reset author details</button
      >
    </div>
  {/if}
  {#if error}<p role="alert" class="ka-error">{error}</p>{/if}
</div>
