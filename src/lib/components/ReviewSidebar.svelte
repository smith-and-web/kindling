<script lang="ts">
  import { tick, type Snippet } from "svelte";
  import {
    ChevronUp,
    ChevronDown,
    MessageSquare,
    Check,
    X,
    MoreHorizontal,
    TriangleAlert,
  } from "lucide-svelte";
  import type { ReviewItem } from "../utils/reviewItems";
  import { holdLinkClick } from "../utils/safeHtml";
  let {
    items,
    selected,
    filter = $bindable("open"),
    name,
    onName,
    onSelect,
    onStep,
    onReply,
    onDecide,
    onWithdraw,
    onResolve,
    canReanchor = false,
    busy = false,
    composing = false,
    comment = $bindable(""),
    onComment,
    onCancelComment,
    options,
    references,
  }: {
    items: ReviewItem[];
    selected: string | null;
    filter?: string;
    name: string;
    onName: (name: string) => void;
    onSelect: (id: string) => void;
    onStep: (direction: number) => void;
    onReply: (id: string, text: string) => Promise<boolean>;
    onDecide: (id: string, decision: string, reanchor?: boolean) => void;
    onWithdraw: (id: string) => void;
    onResolve: (id: string) => void;
    canReanchor?: boolean;
    busy?: boolean;
    composing?: boolean;
    comment?: string;
    onComment: () => void;
    onCancelComment: () => void;
    options?: Snippet;
    references?: Snippet;
  } = $props();
  let tab = $state("review");
  let enteringName = $state(false);
  $effect(() => {
    if (!name.trim()) enteringName = true;
  });
  let reply = $state("");
  let container: HTMLElement;
  let optionsMenu = $state<HTMLDetailsElement>();
  function dismissOptionsOutside(event: MouseEvent) {
    if (optionsMenu?.open && event.target instanceof Node && !optionsMenu.contains(event.target))
      optionsMenu.open = false;
  }
  function dismissOptions(event: MouseEvent) {
    const target = event.target;
    if (
      optionsMenu?.open &&
      target instanceof Element &&
      (!optionsMenu.contains(target) || target.closest(".review-menu button:not(:disabled)"))
    )
      optionsMenu.open = false;
  }
  function escapeOptions(event: KeyboardEvent) {
    if (event.key === "Escape" && optionsMenu?.open) {
      event.preventDefault();
      event.stopPropagation();
      optionsMenu.open = false;
      optionsMenu.querySelector("summary")?.focus();
    }
  }
  const visible = $derived(items.filter((i) => filter === "all" || i.state === filter));
  const tabs = $derived(references ? ["review", "references"] : ["review"]);
  // Tabs: roving focus with arrows, Home and End.
  function onTabKeydown(event: KeyboardEvent) {
    const index = tabs.indexOf(tab);
    const moves: Record<string, number> = {
      ArrowRight: (index + 1) % tabs.length,
      ArrowLeft: (index - 1 + tabs.length) % tabs.length,
      Home: 0,
      End: tabs.length - 1,
    };
    const next = moves[event.key];
    if (next === undefined) return;
    event.preventDefault();
    tab = tabs[next];
    container?.querySelector<HTMLElement>(`#review-tab-${tabs[next]}`)?.focus();
  }
  const stateLabels: Record<string, string> = {
    resolved: "Resolved",
    accepted: "Accepted",
    rejected: "Rejected",
    withdrawn: "Withdrawn",
  };
  // The badge text carries the meaning; tone only reinforces it.
  function badge(item: ReviewItem) {
    if (item.unavailable) return { label: "Inactive prose", tone: "ka-badge--warning" };
    if (item.state === "open")
      return { label: item.kind === "comment" ? "Comment" : "Suggested edit", tone: "" };
    return {
      label: stateLabels[item.state] ?? item.state,
      tone: item.state === "accepted" ? "ka-badge--success" : "",
    };
  }
  function shortDate(value?: string) {
    if (!value) return "";
    const date = new Date(value);
    return Number.isNaN(date.getTime())
      ? ""
      : date.toLocaleDateString(undefined, { month: "short", day: "numeric" });
  }
  // The passage the feedback is anchored to, quoted above the note it carries.
  function anchor(item: ReviewItem) {
    return item.messages[0]?.text && item.excerpt && item.excerpt !== "General feedback"
      ? item.excerpt
      : "";
  }
  $effect(() => {
    if (selected || composing) {
      tab = "review";
      reply = "";
      void tick().then(() => {
        if (composing)
          container?.querySelector<HTMLTextAreaElement>('[aria-label="Comment"]')?.focus();
        else
          container?.querySelector('[data-selected="true"]')?.scrollIntoView({ block: "nearest" });
      });
    }
  });
</script>

<svelte:window
  onpointerdowncapture={dismissOptionsOutside}
  onclickcapture={dismissOptionsOutside}
  onclick={(event) => {
    dismissOptions(event);
    // Suggested passages render the reviewer's links; following one would replace the app.
    if (event.target instanceof Node && container?.contains(event.target)) holdLinkClick(event);
  }}
/>

<aside class="review-sidebar" bind:this={container} aria-label="Editorial feedback">
  <div class="ka-tablist review-tabs" role="tablist" aria-label="Inspector">
    <button
      type="button"
      role="tab"
      id="review-tab-review"
      aria-selected={tab === "review"}
      aria-controls="review-tabpanel"
      tabindex={tab === "review" ? 0 : -1}
      onclick={() => (tab = "review")}
      onkeydown={onTabKeydown}
      >Review <span class="tab-count">{items.filter((i) => i.state === "open").length}</span
      ></button
    >
    {#if references}<button
        type="button"
        role="tab"
        id="review-tab-references"
        aria-selected={tab === "references"}
        aria-controls="review-tabpanel"
        tabindex={tab === "references" ? 0 : -1}
        onclick={() => (tab = "references")}
        onkeydown={onTabKeydown}>References</button
      >{/if}
  </div>
  <div
    class="review-tabpanel"
    id="review-tabpanel"
    role="tabpanel"
    aria-labelledby={`review-tab-${tab}`}
  >
    {#if tab === "references"}{@render references?.()}{:else}
      <div class="review-controls">
        <div class="ka-field od-field show-field">
          <label for="review-filter">Show</label>
          <select id="review-filter" aria-label="Show feedback" bind:value={filter}
            ><option value="open">Pending feedback</option><option value="all">All feedback</option
            ><option value="resolved">Resolved comments</option><option value="accepted"
              >Accepted</option
            ><option value="rejected">Rejected</option></select
          >
        </div>
        <button
          type="button"
          class="ka-button ka-button--ghost ka-icon-button"
          title="Previous feedback"
          aria-label="Previous feedback"
          disabled={!visible.length}
          onclick={() => onStep(-1)}><ChevronUp class="w-5 h-5" aria-hidden="true" /></button
        >
        <button
          type="button"
          class="ka-button ka-button--ghost ka-icon-button"
          title="Next feedback"
          aria-label="Next feedback"
          disabled={!visible.length}
          onclick={() => onStep(1)}><ChevronDown class="w-5 h-5" aria-hidden="true" /></button
        >
        <details class="review-options" bind:this={optionsMenu} onkeydowncapture={escapeOptions}>
          <summary aria-label="Review options" title="Review options"
            ><MoreHorizontal class="w-5 h-5" aria-hidden="true" /></summary
          >
          <div class="review-menu">
            <div class="ka-field od-field menu-identity">
              <label for="review-menu-name">Your name</label>
              <input
                id="review-menu-name"
                required
                aria-invalid={!name.trim()}
                value={name}
                oninput={(e) => onName(e.currentTarget.value)}
              />
            </div>
            {@render options?.()}
          </div>
        </details>
      </div>
      {#if enteringName}<div class="identity">
          <div class="ka-field od-field">
            <label for="review-name">Name shown with feedback</label>
            <input
              id="review-name"
              required
              aria-invalid={!name.trim()}
              aria-describedby="review-name-help"
              value={name}
              oninput={(e) => onName(e.currentTarget.value)}
              placeholder="Your name"
            />
            <p id="review-name-help" class="ka-help">
              Enter your name to add comments and export feedback.
            </p>
          </div>
          <div>
            <button
              type="button"
              class="ka-button ka-button--secondary"
              disabled={!name.trim()}
              onclick={() => (enteringName = false)}>Done</button
            >
          </div>
        </div>{/if}
      <div class="threads">
        {#if composing}<section class="comment-compose">
            <div class="ka-field od-field">
              <label for="review-comment">Comment</label>
              <textarea
                id="review-comment"
                aria-label="Comment"
                bind:value={comment}
                rows="4"
                placeholder="What would you like the writer to consider?"
              ></textarea>
            </div>
            <div class="actions">
              <button
                type="button"
                class="ka-button"
                disabled={busy || !name.trim() || !comment.trim()}
                onclick={onComment}>Save comment</button
              ><button
                type="button"
                class="ka-button ka-button--secondary"
                onclick={onCancelComment}>Cancel</button
              >
            </div>
          </section>{/if}
        {#each visible as item (item.id)}
          {@const status = badge(item)}
          {@const quote = anchor(item)}
          {@const date = shortDate(item.messages[0]?.created_at)}
          <article
            class="feedback-card"
            data-selected={selected === item.id}
            class:selected={selected === item.id}
            aria-current={selected === item.id ? "true" : undefined}
          >
            <button
              type="button"
              class="thread-heading"
              aria-pressed={selected === item.id}
              onclick={() => onSelect(item.id)}
            >
              <span class="card-head"
                ><span class="author"
                  >{item.author || "Review"}{#if date}<span class="card-date">{` · ${date}`}</span
                    >{/if}</span
                ><span class="ka-badge {status.tone}">{status.label}</span></span
              >
              {#if quote}<q class="anchor">{quote}</q>{/if}
              {#if selected !== item.id || !item.messages.length}<span class="excerpt"
                  >{item.messages[0]?.text || item.excerpt || "Formatting change"}</span
                >{/if}
            </button>
            {#if selected === item.id}
              <div class="card-body">
                {#if item.kind === "suggestion"}<div class="comparison">
                    {#if item.conflict}<h3>Original passage</h3>{/if}
                    <div class="original-prose">{@html item.before || "Insertion point"}</div>
                    {#if item.conflict}<h3>Current passage</h3>
                      <p>{item.current || "Empty passage"}</p>
                      <h3>Suggested passage</h3>{/if}
                    <div class="suggested-prose">{@html item.after || "Delete passage"}</div>
                  </div>{/if}
                {#if item.conflict && item.state === "open"}<div
                    class="ka-notice ka-notice--warning od-row-top conflict"
                  >
                    <TriangleAlert class="w-5 h-5" aria-hidden="true" />
                    <p class="od-fill">
                      {item.kind === "suggestion" || item.reanchor
                        ? "This passage has changed. Select where this feedback belongs in the manuscript."
                        : "This passage has changed since the review. The original discussion remains available below."}
                    </p>
                  </div>{/if}
                {#each item.messages as note}<div class="message">
                    <span class="message-meta"
                      ><span class="message-author">{note.author}</span><time
                        >{new Date(note.created_at).toLocaleDateString()}</time
                      ></span
                    >
                    <p>{note.text}</p>
                  </div>{/each}
                {#if item.unavailable}<p class="ka-help">
                    {item.unavailable}
                  </p>{:else if item.locked}<p class="ka-help">
                    Unlock this scene to change its review.
                  </p>{:else}
                  <div class="ka-field od-field reply">
                    <label for="review-reply">Reply</label>
                    <textarea
                      id="review-reply"
                      aria-label="Reply"
                      bind:value={reply}
                      rows="2"
                      placeholder="Reply to this conversation…"
                    ></textarea>
                  </div>
                  <div class="actions">
                    <button
                      type="button"
                      class="ka-button ka-button--secondary"
                      disabled={busy || !reply.trim() || !name.trim()}
                      onclick={async () => {
                        const text = reply;
                        if (await onReply(item.id, text)) {
                          if (selected === item.id && reply === text) reply = "";
                        }
                      }}>Reply</button
                    >
                    {#if item.resolve}<button
                        type="button"
                        class="ka-button ka-button--ghost"
                        disabled={busy}
                        onclick={() => onResolve(item.id)}
                        >{item.state === "resolved" ? "Reopen thread" : "Resolve thread"}</button
                      >{/if}
                  </div>
                  {#if item.decide && item.kind === "suggestion" && item.state === "open"}<div
                      class="actions decisions"
                    >
                      <button
                        type="button"
                        class="ka-button ka-button--secondary"
                        disabled={busy || item.conflict}
                        onclick={() => onDecide(item.id, "accepted")}
                        ><Check class="w-5 h-5" aria-hidden="true" />Accept</button
                      >
                      <button
                        type="button"
                        class="ka-button ka-button--ghost"
                        disabled={busy}
                        onclick={() => onDecide(item.id, "rejected")}
                        ><X class="w-5 h-5" aria-hidden="true" />Reject</button
                      >
                      {#if item.conflict}<button
                          type="button"
                          class="ka-button ka-button--secondary"
                          disabled={busy || !canReanchor}
                          onclick={() => onDecide(item.id, "accepted", true)}
                          >Apply to selected passage</button
                        >{/if}
                    </div>{/if}
                  {#if item.conflict && item.kind === "comment" && item.reanchor}<div
                      class="actions"
                    >
                      <button
                        type="button"
                        class="ka-button ka-button--secondary"
                        disabled={busy || !canReanchor}
                        onclick={() => onDecide(item.id, "reanchor", true)}
                        >Re-anchor to selection</button
                      >
                    </div>{/if}
                  {#if item.withdraw}<div class="actions">
                      <button
                        type="button"
                        class="ka-button ka-button--ghost"
                        disabled={busy}
                        onclick={() => onWithdraw(item.id)}>Withdraw {item.kind}</button
                      >
                    </div>{/if}
                {/if}
              </div>
            {/if}
          </article>
        {/each}
        {#if !visible.length && !composing}<div class="ka-empty od-stack review-empty">
            <MessageSquare class="w-7 h-7" aria-hidden="true" />
            <h3>No feedback in this view</h3>
            <p>Select a passage to comment, or use Suggesting to propose an edit.</p>
          </div>{/if}
      </div>
    {/if}
  </div>
</aside>

<style>
  /* Press feedback panel: tabs, a filter bar, hairline-separated cards. */
  .review-sidebar {
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    width: 360px;
    min-height: 0;
    border-left: var(--border-hair);
    background: var(--color-bg);
    color: var(--color-text);
    font: var(--text-ui) / 1.5 var(--font-ui);
  }
  @media (max-width: 1280px) {
    .review-sidebar {
      width: 320px;
    }
  }
  .review-tabs {
    flex: none;
    flex-wrap: nowrap;
    gap: var(--space-3xs);
    padding: 0 var(--space-s);
  }
  .review-tabs > button {
    white-space: nowrap;
  }
  .tab-count {
    margin-left: var(--space-3xs);
    font-weight: 400;
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
  }
  .review-tabs > button[aria-selected="true"] .tab-count {
    color: inherit;
  }
  .review-tabpanel {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }

  .review-controls {
    display: flex;
    align-items: flex-end;
    gap: var(--space-3xs);
    padding: var(--space-s);
    border-bottom: var(--border-hair);
  }
  .show-field {
    flex: 1;
    min-width: 0;
  }
  .show-field select {
    padding-block: var(--space-2xs);
    font-size: var(--text-ui);
  }
  .review-options {
    position: relative;
    flex: none;
  }
  .review-options > summary {
    display: flex;
    align-items: center;
    justify-content: center;
    width: var(--control-target);
    height: var(--control-target);
    border-radius: var(--radius-xs);
    color: var(--color-text);
    list-style: none;
    cursor: pointer;
  }
  .review-options > summary::-webkit-details-marker {
    display: none;
  }
  .review-options[open] > summary {
    background: var(--color-surface-sunken);
  }
  @media (hover: hover) {
    .review-options > summary:hover {
      background: var(--color-surface-sunken);
    }
  }
  /* The options menu is a floating layer: surface, hairline, overlay shadow. */
  .review-menu {
    position: absolute;
    right: 0;
    top: calc(100% + var(--space-3xs));
    z-index: var(--z-dropdown);
    width: 20rem;
    max-width: calc(100vw - var(--space-l));
    max-height: 70vh;
    overflow: auto;
    padding: var(--space-2xs);
    border: var(--border-hair);
    border-radius: var(--radius-m);
    background: var(--color-surface);
    box-shadow: var(--shadow-overlay);
  }
  .menu-identity {
    padding: var(--space-2xs) var(--space-2xs) var(--space-xs);
  }
  .review-menu :global(.review-menu-group) {
    display: grid;
    gap: var(--space-3xs);
    margin-top: var(--space-2xs);
    padding-top: var(--space-2xs);
    border-top: var(--border-hair);
  }
  .review-menu :global(.review-menu-caption) {
    margin: 0;
    padding: var(--space-3xs) var(--space-xs);
    overflow-wrap: anywhere;
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .review-menu :global(.review-menu-field) {
    padding: var(--space-2xs);
  }
  .review-menu :global(.review-menu-group > button) {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    width: 100%;
    min-height: var(--control-target);
    padding: var(--space-2xs) var(--space-xs);
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--color-text);
    font: var(--text-ui) / 1.4 var(--font-ui);
    text-align: left;
    cursor: pointer;
  }
  .review-menu :global(.review-menu-group > button:disabled) {
    background: transparent !important;
    border-color: transparent !important;
  }
  .review-menu :global(.review-menu-group > button:hover:not(:disabled)) {
    background: var(--color-surface-sunken);
  }
  .review-menu :global(.review-menu-group > button.accept-decision:hover:not(:disabled)) {
    background: var(--color-success-wash);
  }
  .review-menu :global(.review-menu-group > button.reject-decision:hover:not(:disabled)) {
    background: var(--color-error-wash);
  }
  .review-menu :global(.review-menu-group > button:focus-visible) {
    outline-offset: -2px;
  }
  .review-menu :global(.review-menu-group > button svg) {
    flex-shrink: 0;
  }

  .identity,
  .comment-compose {
    display: grid;
    gap: var(--space-xs);
    padding: var(--space-s);
    border-bottom: var(--border-hair);
  }
  .threads {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  /* A feedback card: hairline separated; the current one is washed and ringed. */
  .feedback-card {
    border-bottom: var(--border-hair);
  }
  .feedback-card.selected {
    background: var(--color-accent-wash);
    box-shadow: inset 0 0 0 1px var(--color-accent-text);
  }
  .thread-heading {
    display: grid;
    gap: var(--space-2xs);
    width: 100%;
    padding: var(--space-s);
    border: 0;
    border-radius: 0;
    background: transparent;
    color: var(--color-text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .thread-heading:focus-visible {
    outline-offset: -3px;
  }
  @media (hover: hover) {
    .feedback-card:not(.selected) .thread-heading:hover {
      background: var(--color-surface-sunken);
    }
  }
  .card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2xs);
    min-width: 0;
  }
  /* Who and when is metadata; the badge and the note carry the card. */
  .author {
    min-width: 0;
    overflow-wrap: anywhere;
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .card-date {
    white-space: nowrap;
  }
  /* A narrow panel gives the filter its own row so its value never clips. */
  @media (max-width: 1280px) {
    .review-controls {
      flex-wrap: wrap;
      justify-content: flex-end;
    }
    .show-field {
      flex-basis: 100%;
    }
  }
  .card-date,
  .message-meta time {
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .card-head .ka-badge {
    flex: none;
  }
  .anchor {
    display: block;
    padding-left: var(--space-xs);
    border-left: var(--border-hair);
    overflow-wrap: anywhere;
    quotes: none;
    font: italic var(--text-ui) / 1.5 var(--font-body);
    color: var(--color-text-muted);
  }
  .excerpt {
    display: block;
    overflow-wrap: anywhere;
    white-space: pre-wrap;
    font: var(--text-ui) / 1.55 var(--font-ui);
  }
  .card-body {
    display: grid;
    gap: var(--space-xs);
    padding: 0 var(--space-s) var(--space-s);
  }
  .comparison {
    display: grid;
    gap: var(--space-3xs);
    overflow-wrap: anywhere;
    font: var(--text-ui) / 1.5 var(--font-body);
  }
  .comparison h3 {
    margin: var(--space-2xs) 0 0;
    font: 600 var(--text-small) / 1.4 var(--font-ui);
    letter-spacing: 0;
    color: var(--color-text);
  }
  .comparison p {
    margin: 0;
  }
  /* Strike and underline carry the meaning; colour only reinforces it. */
  .original-prose {
    color: var(--color-error);
    text-decoration: line-through;
  }
  .suggested-prose {
    color: var(--color-success);
    text-decoration: underline;
    text-underline-offset: 3px;
  }
  .message {
    display: grid;
    gap: var(--space-3xs);
  }
  .message-meta {
    display: flex;
    justify-content: space-between;
    gap: var(--space-2xs);
  }
  .message-author {
    font: 500 var(--text-ui) / 1.4 var(--font-ui);
  }
  .message p {
    margin: 0;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    font: var(--text-ui) / 1.55 var(--font-ui);
  }
  .conflict p {
    margin: 0;
  }
  .reply textarea {
    min-height: 88px;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2xs);
  }
  .review-empty {
    padding: var(--space-l) var(--space-s);
  }
  .review-empty h3 {
    margin: 0;
    font: 550 var(--text-h3) / 1.25 var(--font-display);
    letter-spacing: var(--tracking-tight);
    color: var(--color-text);
  }
  .review-empty p {
    font-size: var(--text-ui);
  }
</style>
