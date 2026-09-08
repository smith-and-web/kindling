<script lang="ts">
  import { tick, type Snippet } from "svelte";
  import { ChevronUp, ChevronDown, MessageSquare, Check, X, MoreHorizontal } from "lucide-svelte";
  import type { ReviewItem } from "../utils/reviewItems";
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

<svelte:window onclick={dismissOptions} />

<aside class="review-sidebar" bind:this={container} aria-label="Editorial feedback">
  <div class="tabs" role="tablist" aria-label="Inspector">
    <button role="tab" aria-selected={tab === "review"} onclick={() => (tab = "review")}
      >Review <span>{items.filter((i) => i.state === "open").length}</span></button
    >
    {#if references}<button
        role="tab"
        aria-selected={tab === "references"}
        onclick={() => (tab = "references")}>References</button
      >{/if}
  </div>
  {#if tab === "references"}{@render references?.()}{:else}
    <div class="review-controls">
      <span class="compact-select"
        ><select aria-label="Show feedback" bind:value={filter}
          ><option value="open">Pending feedback</option><option value="all">All feedback</option
          ><option value="resolved">Resolved comments</option><option value="accepted"
            >Accepted</option
          ><option value="rejected">Rejected</option></select
        ><ChevronDown size={14} /></span
      >
      <button
        title="Previous annotation"
        aria-label="Previous annotation"
        disabled={!visible.length}
        onclick={() => onStep(-1)}><ChevronUp size={16} /></button
      >
      <button
        title="Next annotation"
        aria-label="Next annotation"
        disabled={!visible.length}
        onclick={() => onStep(1)}><ChevronDown size={16} /></button
      >
      <details bind:this={optionsMenu} onkeydowncapture={escapeOptions}>
        <summary aria-label="Review options"><MoreHorizontal size={18} /></summary>
        <div class="review-menu">
          <label class="menu-identity"
            >Your name<input value={name} oninput={(e) => onName(e.currentTarget.value)} /></label
          >
          {@render options?.()}
        </div>
      </details>
    </div>
    {#if enteringName}<div class="identity">
        <label
          >Name shown with feedback<input
            value={name}
            oninput={(e) => onName(e.currentTarget.value)}
            placeholder="Your name"
          /></label
        >
        <button disabled={!name.trim()} onclick={() => (enteringName = false)}>Done</button>
      </div>{/if}
    <div class="threads">
      {#if composing}<section class="compose">
          <label
            >Comment<textarea
              aria-label="Comment"
              bind:value={comment}
              rows="4"
              placeholder="What would you like the writer to consider?"
            ></textarea></label
          >
          <div class="actions">
            <button disabled={busy || !name.trim() || !comment.trim()} onclick={onComment}
              >Save comment</button
            ><button onclick={onCancelComment}>Cancel</button>
          </div>
        </section>{/if}
      {#each visible as item (item.id)}
        <article data-selected={selected === item.id} class:selected={selected === item.id}>
          <button
            class="thread-heading"
            aria-pressed={selected === item.id}
            onclick={() => onSelect(item.id)}
          >
            <span class="author"
              >{item.author || "Review"}<span class="state"
                >{item.unavailable
                  ? "Inactive prose"
                  : item.state === "open"
                    ? item.kind === "comment"
                      ? "Comment"
                      : "Suggested edit"
                    : item.state}</span
              ></span
            >
            {#if selected !== item.id || !item.messages.length}<span class="excerpt"
                >{item.messages[0]?.text || item.excerpt || "Formatting change"}</span
              >{/if}
          </button>
          {#if selected === item.id}
            {#if item.kind === "suggestion"}<div class="comparison">
                {#if item.conflict}<h3>Original passage</h3>{/if}
                <div class="original-prose">{@html item.before || "Insertion point"}</div>
                {#if item.conflict}<h3>Current passage</h3>
                  <p>{item.current || "Empty passage"}</p>
                  <h3>Suggested passage</h3>{/if}
                <div class="suggested-prose">{@html item.after || "Delete passage"}</div>
              </div>{/if}
            {#if item.conflict && item.state === "open"}<p class="conflict">
                {item.kind === "suggestion" || item.reanchor
                  ? "This passage has changed. Select where this feedback belongs in the manuscript."
                  : "This passage has changed since the review. The original discussion remains available below."}
              </p>{/if}
            {#each item.messages as note}<div class="message">
                <span class="author"
                  >{note.author}<time>{new Date(note.created_at).toLocaleDateString()}</time></span
                >
                <p>{note.text}</p>
              </div>{/each}
            {#if item.unavailable}<p class="hint">{item.unavailable}</p>{:else if item.locked}<p
                class="hint"
              >
                Unlock this scene to change its review.
              </p>{:else}
              <label class="reply"
                >Reply<textarea
                  aria-label="Reply"
                  bind:value={reply}
                  rows="2"
                  placeholder="Reply to this conversation…"
                ></textarea></label
              >
              <div class="actions">
                <button
                  disabled={busy || !reply.trim() || !name.trim()}
                  onclick={async () => {
                    const text = reply;
                    if (await onReply(item.id, text)) {
                      if (selected === item.id && reply === text) reply = "";
                    }
                  }}>Reply</button
                >
                {#if item.resolve}<button disabled={busy} onclick={() => onResolve(item.id)}
                    >{item.state === "resolved" ? "Reopen thread" : "Resolve thread"}</button
                  >{/if}
              </div>
              {#if item.decide && item.kind === "suggestion" && item.state === "open"}<div
                  class="actions decisions"
                >
                  <button
                    class="accept-decision"
                    disabled={busy || item.conflict}
                    onclick={() => onDecide(item.id, "accepted")}><Check size={16} />Accept</button
                  >
                  <button
                    class="reject-decision"
                    disabled={busy}
                    onclick={() => onDecide(item.id, "rejected")}><X size={16} />Reject</button
                  >
                  {#if item.conflict}<button
                      disabled={busy || !canReanchor}
                      onclick={() => onDecide(item.id, "accepted", true)}
                      >Apply to selected passage</button
                    >{/if}
                </div>{/if}
              {#if item.conflict && item.kind === "comment" && item.reanchor}<button
                  disabled={busy || !canReanchor}
                  onclick={() => onDecide(item.id, "reanchor", true)}>Re-anchor to selection</button
                >{/if}
              {#if item.withdraw}<button
                  class="quiet"
                  disabled={busy}
                  onclick={() => onWithdraw(item.id)}>Withdraw {item.kind}</button
                >{/if}
            {/if}
          {/if}
        </article>
      {/each}
      {#if !visible.length && !composing}<div class="empty">
          <MessageSquare size={24} />
          <p>No feedback in this view.</p>
          <p>Select a passage to comment, or use Suggesting to propose an edit.</p>
        </div>{/if}
    </div>
  {/if}
</aside>

<style>
  .review-sidebar {
    width: 20rem;
    flex-shrink: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    border-left: 1px solid var(--color-border);
    background: var(--color-surface);
    font-family: var(--font-ui);
    font-size: var(--text-small);
    color: var(--color-text);
  }
  .tabs {
    display: flex;
    border-bottom: 1px solid var(--color-border);
    padding-inline: var(--space-s);
    gap: var(--space-s);
  }
  .tabs button {
    padding: var(--space-s) var(--space-2xs);
    border: 0;
    border-bottom: 2px solid transparent;
    border-radius: 0;
  }
  .tabs button[aria-selected="true"] {
    border-bottom-color: var(--color-accent);
    color: var(--color-accent-text);
  }
  .tabs span {
    margin-left: var(--space-2xs);
    color: var(--color-text-muted);
  }
  button {
    background: transparent;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-s);
    color: var(--color-text);
    padding: var(--space-2xs);
    font: inherit;
    cursor: pointer;
  }
  button:hover {
    background: var(--color-surface-sunken);
  }
  button:disabled {
    color: var(--color-disabled-text);
    background: var(--color-disabled-bg);
    cursor: default;
  }
  .review-controls {
    display: flex;
    align-items: center;
    gap: var(--space-3xs);
    padding: var(--space-xs);
    border-bottom: 1px solid var(--color-border);
  }
  .compact-select {
    display: grid;
    align-items: center;
    min-width: 0;
    flex: 1;
  }
  .compact-select select {
    grid-area: 1 / 1;
    appearance: none;
  }
  .compact-select :global(svg) {
    grid-area: 1 / 1;
    justify-self: end;
    margin-right: var(--space-2xs);
    pointer-events: none;
    color: var(--color-text-muted);
  }
  .review-controls button {
    border: 0;
    display: flex;
  }
  details {
    position: relative;
  }
  summary {
    cursor: pointer;
    list-style: none;
    display: flex;
    padding: var(--space-2xs);
  }
  summary::-webkit-details-marker {
    display: none;
  }
  summary:focus-visible {
    outline: none;
    box-shadow: var(--focus-ring);
    border-radius: var(--radius-s);
  }
  .review-menu {
    position: absolute;
    right: 0;
    width: 18rem;
    max-height: 70vh;
    overflow: auto;
    padding: var(--space-2xs);
    background: var(--color-surface);
    box-shadow: var(--shadow-overlay);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-m);
    z-index: var(--z-dropdown);
  }
  .menu-identity {
    padding: var(--space-2xs);
  }
  .review-menu :global(.review-menu-group) {
    border-top: 1px solid var(--color-border);
    margin-top: var(--space-2xs);
    padding-top: var(--space-2xs);
  }
  .review-menu :global(.review-menu-caption) {
    margin: 0;
    padding: var(--space-3xs) var(--space-2xs);
    font-size: var(--text-eyebrow);
    color: var(--color-text-muted);
    overflow-wrap: anywhere;
  }
  .review-menu :global(.review-menu-field) {
    display: flex;
    flex-direction: column;
    gap: var(--space-2xs);
    padding: var(--space-2xs);
  }
  .review-menu :global(button) {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    width: 100%;
    border: 0;
    padding: var(--space-2xs);
    text-align: left;
    line-height: var(--leading-tight);
  }
  .threads {
    min-height: 0;
    overflow: auto;
    flex: 1;
    padding: var(--space-xs);
  }
  article {
    border-bottom: 1px solid var(--color-border);
    padding-block: var(--space-xs);
  }
  article.selected {
    border-left: 2px solid var(--color-accent);
    padding-left: var(--space-xs);
  }
  .thread-heading {
    display: block;
    text-align: left;
    width: 100%;
    border: 0;
    padding: var(--space-2xs);
  }
  .author {
    display: flex;
    justify-content: space-between;
    gap: var(--space-2xs);
    font-weight: 600;
  }
  .state,
  time {
    font-weight: 400;
    font-size: var(--text-eyebrow);
    color: var(--color-text-muted);
  }
  .excerpt {
    display: block;
    margin-top: var(--space-2xs);
    line-height: var(--leading);
  }
  .comparison {
    font-family: var(--font-body);
    font-size: var(--text-body);
    margin: var(--space-xs) var(--space-2xs);
    overflow-wrap: anywhere;
  }
  h3 {
    font-family: var(--font-ui);
    font-size: var(--text-small);
    margin-block: var(--space-xs) var(--space-2xs);
  }
  .original-prose {
    text-decoration: line-through;
  }
  .suggested-prose {
    text-decoration: underline;
  }
  .message {
    margin: var(--space-xs) var(--space-2xs);
  }
  .message p {
    margin-block: var(--space-2xs);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: var(--space-2xs);
  }
  .identity,
  .compose {
    padding: var(--space-xs);
  }
  textarea,
  input,
  select {
    /* Compact desktop review controls, explicitly requested for this workspace. */
    font-size: var(--text-small);
    padding: var(--space-3xs) var(--space-2xs);
    line-height: var(--leading);
    width: 100%;
    box-sizing: border-box;
  }
  .compact-select select {
    padding-right: var(--space-l);
  }
  textarea {
    padding-block: var(--space-2xs);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2xs);
    margin-block: var(--space-2xs);
  }
  .decisions button {
    display: flex;
    align-items: center;
    gap: var(--space-3xs);
  }
  .accept-decision:not(:disabled) {
    color: var(--color-success);
    border-color: var(--color-success);
    background: var(--color-success-wash);
  }
  .reject-decision:not(:disabled) {
    color: var(--color-error);
    border-color: var(--color-error);
    background: var(--color-error-wash);
  }
  .quiet {
    border: 0;
    text-decoration: underline;
    color: var(--color-text-muted);
  }
  .hint,
  .empty {
    color: var(--color-text-muted);
    line-height: var(--leading-relaxed);
  }
  .empty {
    padding: var(--space-l) var(--space-xs);
  }
  .conflict {
    padding: var(--space-xs);
    border-left: 2px solid var(--color-warning);
  }
</style>
