<!--
  FeedbackDialog.svelte - Send feedback to the kindling team

  A small form letting the user submit a bug report, feature request, or star
  rating. Apart from the updater's release check, submission is the only network
  request the app makes, and it is strictly user-initiated: it happens
  exclusively when the user clicks "Send feedback".
  The POST itself is performed on the Rust side via invoke("submit_feedback");
  nothing is persisted locally and offline use is unaffected.
-->
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus } from "../utils/modalFocus";
  import { Loader2, Send, Star, CheckCircle2, AlertCircle } from "lucide-svelte";

  type FeedbackType = "bug" | "feature" | "rating";

  let { onClose }: { onClose: () => void } = $props();

  // Mirrors the backend limits in src-tauri/src/commands/feedback.rs.
  const MAX_SUMMARY_LEN = 120;
  const MAX_MESSAGE_LEN = 2000;

  const FEEDBACK_TYPES: { value: FeedbackType; label: string }[] = [
    { value: "bug", label: "Bug" },
    { value: "feature", label: "Feature" },
    { value: "rating", label: "Rating" },
  ];

  let feedbackType = $state<FeedbackType>("bug");
  let summary = $state("");
  let message = $state("");
  let rating = $state(0);

  let sending = $state(false);
  let status = $state<"idle" | "success" | "error">("idle");
  let error = $state<string | null>(null);
  let validationError = $state<string | null>(null);

  function selectType(value: FeedbackType) {
    feedbackType = value;
    validationError = null;
  }

  function locale(): string {
    return globalThis.navigator?.language ?? "en-US";
  }

  /** Returns a validation message if the form is invalid, otherwise null. */
  function validate(): string | null {
    if (summary.trim().length > MAX_SUMMARY_LEN) {
      return `Summary must be ${MAX_SUMMARY_LEN} characters or fewer.`;
    }
    if (feedbackType === "rating") {
      if (rating < 1 || rating > 5) {
        return "Please select a rating from 1 to 5.";
      }
      return null;
    }
    const trimmed = message.trim();
    if (!trimmed) {
      return "Please enter a message.";
    }
    if (trimmed.length > MAX_MESSAGE_LEN) {
      return `Message must be ${MAX_MESSAGE_LEN} characters or fewer.`;
    }
    return null;
  }

  async function handleSubmit() {
    if (sending) return;

    const invalid = validate();
    if (invalid) {
      validationError = invalid;
      return;
    }
    validationError = null;

    const payload: Record<string, unknown> = {
      feedbackType,
      locale: locale(),
    };
    const trimmedSummary = summary.trim();
    if (trimmedSummary) {
      payload.summary = trimmedSummary;
    }
    if (feedbackType === "rating") {
      payload.rating = rating;
    } else {
      payload.message = message.trim();
    }

    sending = true;
    error = null;
    try {
      // The Rust command takes a single `input` parameter, so the payload must
      // be nested under `input` (Tauri maps invoke arg keys to parameter names).
      await invoke("submit_feedback", { input: payload });
      status = "success";
    } catch (e) {
      status = "error";
      error = e instanceof Error ? e.message : String(e);
    } finally {
      sending = false;
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }
</script>

<!-- Escape is the keyboard equivalent of the backdrop click; modalFocus handles it. -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="dialog-scrim"
  use:modalFocus={{ onEscape: onClose, initialFocus: "[aria-pressed='true']" }}
  onclick={handleBackdropClick}
  role="dialog"
  aria-modal="true"
  aria-labelledby="feedback-dialog-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-narrow dialog-shell" data-testid="feedback-dialog">
    <DialogHeader
      title="Send feedback"
      titleId="feedback-dialog-title"
      {onClose}
      closeLabel="Close"
    />

    {#if status === "success"}
      <div class="ka-dialog-body feedback-success" data-testid="feedback-success" role="status">
        <CheckCircle2 class="w-7 h-7" aria-hidden="true" />
        <h3>Thanks for your feedback!</h3>
        <p>Your message was sent to the kindling team.</p>
      </div>
      <footer class="ka-dialog-footer">
        <button type="button" onclick={onClose} class="ka-button">Done</button>
      </footer>
    {:else}
      <div class="ka-dialog-body feedback-form">
        <fieldset class="ka-segments">
          <legend>What kind of feedback?</legend>
          <div class="ka-segment-track">
            {#each FEEDBACK_TYPES as type (type.value)}
              <button
                type="button"
                onclick={() => selectType(type.value)}
                aria-pressed={feedbackType === type.value}
                class="ka-segment"
                class:ka-selected={feedbackType === type.value}
              >
                {type.label}
              </button>
            {/each}
          </div>
        </fieldset>

        {#if feedbackType === "rating"}
          <fieldset class="feedback-rating">
            <legend>How would you rate kindling?</legend>
            <div class="ka-row">
              {#each [1, 2, 3, 4, 5] as n (n)}
                <button
                  type="button"
                  onclick={() => (rating = n)}
                  aria-label={`${n} star${n === 1 ? "" : "s"}`}
                  aria-pressed={rating === n}
                  class="ka-button ka-button--ghost ka-icon-button feedback-star"
                  class:is-on={n <= rating}
                >
                  <Star class="w-6 h-6" fill={n <= rating ? "currentColor" : "none"} />
                </button>
              {/each}
              {#if rating > 0}<span class="ka-help">{rating} of 5</span>{/if}
            </div>
          </fieldset>
        {:else}
          <div class="ka-field od-field">
            <label for="feedback-summary">
              Summary <span class="ka-optional">(optional)</span>
            </label>
            <input
              id="feedback-summary"
              type="text"
              bind:value={summary}
              disabled={sending}
              placeholder="A short title"
              aria-invalid={summary.trim().length > MAX_SUMMARY_LEN || undefined}
            />
            <div class="ka-field-meta">
              <span
                class="ka-count"
                class:ka-error={summary.trim().length > MAX_SUMMARY_LEN}
                aria-live="polite">{summary.trim().length}/{MAX_SUMMARY_LEN}</span
              >
            </div>
          </div>

          <div class="ka-field od-field">
            <label for="feedback-message">Message</label>
            <textarea
              id="feedback-message"
              rows="5"
              bind:value={message}
              disabled={sending}
              placeholder="Tell us what’s on your mind…"
              aria-invalid={message.trim().length > MAX_MESSAGE_LEN || undefined}
            ></textarea>
            <div class="ka-field-meta">
              <span
                class="ka-count"
                class:ka-error={message.trim().length > MAX_MESSAGE_LEN}
                aria-live="polite">{message.trim().length}/{MAX_MESSAGE_LEN}</span
              >
            </div>
          </div>
        {/if}

        <p class="ka-help" data-testid="feedback-disclosure">
          Sending includes what you enter here plus your kindling version, operating system and
          language. Nothing from your manuscript is attached.
        </p>

        {#if validationError}
          <p class="ka-error" data-testid="feedback-validation" role="alert">
            <AlertCircle class="w-4 h-4 shrink-0" aria-hidden="true" />
            {validationError}
          </p>
        {/if}

        {#if status === "error"}
          <div
            class="ka-notice ka-notice--error od-row-top"
            data-testid="feedback-error"
            role="alert"
          >
            <AlertCircle class="w-5 h-5 shrink-0" aria-hidden="true" />
            <div class="od-field od-fill">
              <strong>Couldn’t send your feedback.</strong>
              {#if error}<p>{error}</p>{/if}
              <p>Your form is kept as you left it.</p>
              <button
                type="button"
                onclick={handleSubmit}
                disabled={sending}
                class="ka-button ka-button--secondary feedback-retry"
              >
                Try again
              </button>
            </div>
          </div>
        {/if}
      </div>

      <footer class="ka-dialog-footer">
        <button
          type="button"
          onclick={onClose}
          disabled={sending}
          class="ka-button ka-button--secondary"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleSubmit}
          disabled={sending}
          aria-busy={sending || undefined}
          class="ka-button"
          data-testid="feedback-submit"
        >
          {#if sending}
            <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
            Sending…
          {:else}
            <Send class="w-5 h-5" aria-hidden="true" />
            Send feedback
          {/if}
        </button>
      </footer>
    {/if}
  </div>
</div>

<style>
  .feedback-form {
    display: grid;
    gap: 20px;
  }
  .feedback-rating {
    margin: 0;
    padding: 0;
    border: 0;
  }
  .feedback-rating legend {
    margin-bottom: var(--space-2xs);
    font: 500 var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .feedback-star {
    color: var(--color-text-muted);
  }
  .feedback-star.is-on {
    color: var(--color-accent-text);
  }
  .feedback-retry {
    justify-self: start;
    margin-top: var(--space-2xs);
  }
  .feedback-success {
    display: grid;
    justify-items: start;
    gap: var(--space-2xs);
    color: var(--color-success);
  }
  .feedback-success h3 {
    margin: var(--space-2xs) 0 0;
    font: 550 var(--text-h3) / 1.25 var(--font-display);
    color: var(--color-text);
  }
  .feedback-success p {
    margin: 0;
    font: var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
</style>
