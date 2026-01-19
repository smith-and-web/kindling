<!--
  KindlingSettingsDialog.svelte - App-wide settings dialog

  Allows users to configure their author information and contact details
  that apply across all projects:
  - Author name
  - Contact address (two lines for international flexibility)
  - Phone and email
-->
<script lang="ts">
  /* eslint-disable no-undef */
  import { invoke } from "@tauri-apps/api/core";
  import { X, Loader2, Settings, User } from "lucide-svelte";
  import type { AppSettings } from "../types";
  import Tooltip from "./Tooltip.svelte";

  let {
    onClose,
    onSave,
  }: {
    onClose: () => void;
    onSave: (settings: AppSettings) => void;
  } = $props();

  // Form state
  let authorName = $state("");
  let addressLine1 = $state("");
  let addressLine2 = $state("");
  let phone = $state("");
  let email = $state("");

  let loading = $state(true);
  let saving = $state(false);
  let error = $state<string | null>(null);

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
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function handleSave() {
    saving = true;
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

      const updatedSettings = await invoke<AppSettings>("update_app_settings", {
        settings,
      });

      onSave(updatedSettings);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    } else if (event.key === "Enter" && (event.metaKey || event.ctrlKey) && !saving && !loading) {
      handleSave();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Backdrop -->
<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  onclick={handleBackdropClick}
  onkeydown={(e) => e.key === "Enter" && handleBackdropClick}
  role="dialog"
  aria-modal="true"
  aria-labelledby="settings-dialog-title"
  tabindex="-1"
>
  <!-- Dialog -->
  <div
    class="bg-bg-panel rounded-lg shadow-xl w-full max-w-lg mx-4 overflow-hidden max-h-[90vh] flex flex-col"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-bg-card flex-shrink-0">
      <div class="flex items-center gap-2">
        <Settings class="w-5 h-5 text-accent" />
        <h2 id="settings-dialog-title" class="text-lg font-medium text-text-primary">
          Kindling Settings
        </h2>
      </div>
      <Tooltip text="Close" position="left">
        <button
          type="button"
          onclick={onClose}
          class="p-1 text-text-secondary hover:text-text-primary transition-colors rounded"
          aria-label="Close"
        >
          <X class="w-5 h-5" />
        </button>
      </Tooltip>
    </div>

    <!-- Content -->
    <div class="p-4 space-y-4 overflow-y-auto flex-1">
      {#if loading}
        <div class="flex items-center justify-center py-12">
          <Loader2 class="w-8 h-8 animate-spin text-accent" />
        </div>
      {:else}
        <p class="text-sm text-text-secondary">
          These settings apply to all your projects. Your contact information will appear on
          manuscript title pages when exporting.
        </p>

        <!-- Section: Author Information -->
        <fieldset>
          <legend class="flex items-center gap-2 text-sm font-medium text-accent mb-3">
            <User class="w-4 h-4" />
            Author Information
          </legend>
          <div class="space-y-3">
            <div>
              <label for="author-name" class="block text-sm text-text-secondary mb-1">
                Author Name
              </label>
              <input
                id="author-name"
                type="text"
                bind:value={authorName}
                placeholder="Your legal name"
                disabled={saving}
                class="w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent disabled:opacity-50"
              />
              <p class="text-xs text-text-secondary mt-1">
                Used in contact info on title pages. Projects can override this with a pen name.
              </p>
            </div>
          </div>
        </fieldset>

        <!-- Section: Contact Information -->
        <fieldset>
          <legend class="block text-sm font-medium text-accent mb-3">Contact Information</legend>
          <p class="text-xs text-text-secondary mb-3">
            Optional details for manuscript title pages. Use any format that works for your country.
          </p>
          <div class="space-y-3">
            <div>
              <label for="address-line1" class="block text-sm text-text-secondary mb-1">
                Address Line 1
              </label>
              <input
                id="address-line1"
                type="text"
                bind:value={addressLine1}
                placeholder="Street address or PO Box"
                disabled={saving}
                class="w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent disabled:opacity-50"
              />
            </div>

            <div>
              <label for="address-line2" class="block text-sm text-text-secondary mb-1">
                Address Line 2
              </label>
              <input
                id="address-line2"
                type="text"
                bind:value={addressLine2}
                placeholder="City, State/Province, Postal Code, Country"
                disabled={saving}
                class="w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent disabled:opacity-50"
              />
            </div>

            <div class="grid grid-cols-2 gap-3">
              <div>
                <label for="phone" class="block text-sm text-text-secondary mb-1"> Phone </label>
                <input
                  id="phone"
                  type="tel"
                  bind:value={phone}
                  placeholder="+1 (555) 123-4567"
                  disabled={saving}
                  class="w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent disabled:opacity-50"
                />
              </div>

              <div>
                <label for="email" class="block text-sm text-text-secondary mb-1"> Email </label>
                <input
                  id="email"
                  type="email"
                  bind:value={email}
                  placeholder="author@email.com"
                  disabled={saving}
                  class="w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent disabled:opacity-50"
                />
              </div>
            </div>
          </div>
        </fieldset>

        <!-- Error Message -->
        {#if error}
          <p class="text-sm text-red-400">{error}</p>
        {/if}
      {/if}
    </div>

    <!-- Footer -->
    <div
      class="flex items-center justify-end gap-2 px-4 py-3 border-t border-bg-card flex-shrink-0"
    >
      <button
        type="button"
        onclick={onClose}
        class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
        disabled={saving}
      >
        Cancel
      </button>
      <button
        type="button"
        onclick={handleSave}
        class="px-4 py-2 text-sm bg-accent text-white rounded-lg hover:bg-accent/80 transition-colors disabled:opacity-50 flex items-center gap-2"
        disabled={saving || loading}
      >
        {#if saving}
          <Loader2 class="w-4 h-4 animate-spin" />
          Saving...
        {:else}
          Save Settings
        {/if}
      </button>
    </div>
  </div>
</div>
