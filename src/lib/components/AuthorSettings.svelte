<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Loader2, User } from "lucide-svelte";
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

<div class="space-y-6" aria-busy={busy}>
  <p class="text-press-ui text-press-muted">
    Your author and contact details apply to manuscript title pages across all projects.
  </p>
  {#if loading}<p role="status">Loading author details…</p>
  {:else if loaded}
    <!-- Section: Author Information -->
    <fieldset>
      <legend class="flex items-center gap-2 text-press-ui font-medium text-press-accent-text mb-3">
        <User class="w-4 h-4" />
        Author Information
      </legend>
      <div class="space-y-3">
        <div>
          <label for="author-name" class="block text-press-ui text-press-muted mb-1">
            Author Name
          </label>
          <input
            id="author-name"
            type="text"
            bind:value={authorName}
            placeholder="Your legal name"
            disabled={saving}
            class="w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2 focus:outline-none focus:border-press-accent"
          />
          <p class="text-press-eyebrow text-press-muted mt-1">
            Used in contact info on title pages. Projects can override this with a pen name.
          </p>
        </div>
      </div>
    </fieldset>

    <!-- Section: Contact Information -->
    <fieldset>
      <legend class="block text-press-ui font-medium text-press-accent-text mb-3"
        >Contact Information</legend
      >
      <p class="text-press-eyebrow text-press-muted mb-3">
        Optional details for manuscript title pages. Use any format that works for your country.
      </p>
      <div class="space-y-3">
        <div>
          <label for="address-line1" class="block text-press-ui text-press-muted mb-1">
            Address Line 1
          </label>
          <input
            id="address-line1"
            type="text"
            bind:value={addressLine1}
            placeholder="Street address or PO Box"
            disabled={saving}
            class="w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2 focus:outline-none focus:border-press-accent"
          />
        </div>

        <div>
          <label for="address-line2" class="block text-press-ui text-press-muted mb-1">
            Address Line 2
          </label>
          <input
            id="address-line2"
            type="text"
            bind:value={addressLine2}
            placeholder="City, State/Province, Postal Code, Country"
            disabled={saving}
            class="w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2 focus:outline-none focus:border-press-accent"
          />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label for="phone" class="block text-press-ui text-press-muted mb-1"> Phone </label>
            <input
              id="phone"
              type="tel"
              bind:value={phone}
              placeholder="+1 (555) 123-4567"
              disabled={saving}
              class="w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2 focus:outline-none focus:border-press-accent"
            />
          </div>

          <div>
            <label for="email" class="block text-press-ui text-press-muted mb-1"> Email </label>
            <input
              id="email"
              type="email"
              bind:value={email}
              placeholder="author@email.com"
              disabled={saving}
              class="w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2 focus:outline-none focus:border-press-accent"
            />
          </div>
        </div>
      </div>
    </fieldset>

    <button
      type="button"
      onclick={handleSave}
      disabled={saving}
      class="px-4 py-2 bg-press-accent text-press-on-accent rounded-lg"
      >{#if saving}<Loader2 class="w-4 h-4 animate-spin" />{:else}Save author details{/if}</button
    >
    {#if saved && !dirty}<p role="status" class="text-press-ui text-press-muted">
        Author details saved.
      </p>{/if}
  {:else}<button type="button" onclick={loadSettings}>Retry loading author details</button>{/if}
  {#if error}<p role="alert" class="text-press-ui text-press-error">{error}</p>{/if}
</div>
