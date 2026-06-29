<!--
  FeedbackDialog.svelte - Send feedback to the Kindling team

  A small form letting the user submit a bug report, feature request, or star
  rating. Submission is the ONLY network call the app makes and is strictly
  user-initiated: it happens exclusively when the user clicks "Send feedback".
  The POST itself is performed on the Rust side via invoke("submit_feedback");
  nothing is persisted locally and offline use is unaffected.
-->
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { X, Loader2, Send, Star, CheckCircle2, AlertCircle } from "lucide-svelte";

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

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
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
  onkeydown={handleKeydown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="feedback-dialog-title"
  tabindex="-1"
>
  <!-- Dialog -->
  <div
    class="bg-bg-panel rounded-lg shadow-xl w-full max-w-md mx-4 overflow-hidden"
    data-testid="feedback-dialog"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-5 py-4 border-b border-bg-card">
      <h2 id="feedback-dialog-title" class="text-lg font-heading font-medium text-text-primary">
        Send feedback
      </h2>
      <button
        type="button"
        onclick={onClose}
        class="p-1 rounded hover:bg-bg-card text-text-secondary hover:text-text-primary transition-colors"
        aria-label="Close"
      >
        <X class="w-5 h-5" />
      </button>
    </div>

    {#if status === "success"}
      <!-- Success state -->
      <div
        class="p-6 flex flex-col items-center text-center gap-3"
        data-testid="feedback-success"
        role="status"
      >
        <div class="w-12 h-12 rounded-full bg-green-500/15 flex items-center justify-center">
          <CheckCircle2 class="w-7 h-7 text-green-400" />
        </div>
        <p class="text-base font-medium text-text-primary">Thanks for your feedback!</p>
        <p class="text-sm text-text-secondary">Your message was sent to the Kindling team.</p>
        <button
          type="button"
          onclick={onClose}
          class="mt-2 px-5 py-2 text-sm font-medium bg-accent text-white rounded-lg hover:bg-accent/90 transition-colors"
        >
          Done
        </button>
      </div>
    {:else}
      <!-- Form -->
      <div class="p-5 space-y-4">
        <!-- Feedback type -->
        <fieldset>
          <legend class="block text-sm font-medium text-text-secondary mb-2">
            What kind of feedback?
          </legend>
          <div class="grid grid-cols-3 gap-2">
            {#each FEEDBACK_TYPES as type (type.value)}
              <button
                type="button"
                onclick={() => selectType(type.value)}
                aria-pressed={feedbackType === type.value}
                class="px-3 py-2 text-sm rounded-lg border transition-colors {feedbackType ===
                type.value
                  ? 'border-accent bg-accent/10 text-text-primary'
                  : 'border-bg-card bg-bg-card/50 text-text-secondary hover:text-text-primary'}"
              >
                {type.label}
              </button>
            {/each}
          </div>
        </fieldset>

        {#if feedbackType === "rating"}
          <!-- Rating -->
          <fieldset>
            <legend class="block text-sm font-medium text-text-secondary mb-2">
              How would you rate Kindling?
            </legend>
            <div class="flex items-center gap-1">
              {#each [1, 2, 3, 4, 5] as n (n)}
                <button
                  type="button"
                  onclick={() => (rating = n)}
                  aria-label={`${n} star${n === 1 ? "" : "s"}`}
                  aria-pressed={rating === n}
                  class="p-1 transition-colors {n <= rating
                    ? 'text-accent'
                    : 'text-text-secondary hover:text-text-primary'}"
                >
                  <Star class="w-7 h-7" fill={n <= rating ? "currentColor" : "none"} />
                </button>
              {/each}
            </div>
          </fieldset>
        {:else}
          <!-- Summary (optional) -->
          <div>
            <label
              for="feedback-summary"
              class="block text-sm font-medium text-text-secondary mb-1"
            >
              Summary <span class="text-text-secondary/70">(optional)</span>
            </label>
            <input
              id="feedback-summary"
              type="text"
              bind:value={summary}
              disabled={sending}
              placeholder="A short title"
              class="w-full bg-bg-card text-text-primary text-sm border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent disabled:opacity-50"
            />
            <p
              class="mt-1 text-xs text-right {summary.trim().length > MAX_SUMMARY_LEN
                ? 'text-red-400'
                : 'text-text-secondary/70'}"
            >
              {summary.trim().length}/{MAX_SUMMARY_LEN}
            </p>
          </div>

          <!-- Message (required) -->
          <div>
            <label
              for="feedback-message"
              class="block text-sm font-medium text-text-secondary mb-1"
            >
              Message
            </label>
            <textarea
              id="feedback-message"
              rows="4"
              bind:value={message}
              disabled={sending}
              placeholder="Tell us what's on your mind..."
              class="w-full bg-bg-card text-text-primary text-sm border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent disabled:opacity-50 resize-none"
            ></textarea>
            <p
              class="mt-1 text-xs text-right {message.trim().length > MAX_MESSAGE_LEN
                ? 'text-red-400'
                : 'text-text-secondary/70'}"
            >
              {message.trim().length}/{MAX_MESSAGE_LEN}
            </p>
          </div>
        {/if}

        <!-- Validation error -->
        {#if validationError}
          <p class="text-sm text-red-400" data-testid="feedback-validation" role="alert">
            {validationError}
          </p>
        {/if}

        <!-- Submission error (retryable) -->
        {#if status === "error"}
          <div
            class="p-3 bg-red-500/10 border border-red-500/20 rounded-lg flex items-start gap-2"
            data-testid="feedback-error"
            role="alert"
          >
            <AlertCircle class="w-4 h-4 text-red-400 shrink-0 mt-0.5" />
            <div class="flex-1">
              <p class="text-sm text-red-400">
                Couldn't send your feedback{error ? `: ${error}` : "."}
              </p>
              <button
                type="button"
                onclick={handleSubmit}
                disabled={sending}
                class="mt-2 text-sm font-medium text-accent hover:underline disabled:opacity-50"
              >
                Try again
              </button>
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-end gap-3 px-5 py-4 border-t border-bg-card">
        <button
          type="button"
          onclick={onClose}
          disabled={sending}
          class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors rounded-lg hover:bg-bg-card disabled:opacity-50"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleSubmit}
          disabled={sending}
          class="px-5 py-2 text-sm font-medium bg-accent text-white rounded-lg hover:bg-accent/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          data-testid="feedback-submit"
        >
          {#if sending}
            <Loader2 class="w-4 h-4 animate-spin" />
            Sending...
          {:else}
            <Send class="w-4 h-4" />
            Send feedback
          {/if}
        </button>
      </div>
    {/if}
  </div>
</div>
