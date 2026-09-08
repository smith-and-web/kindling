import { invoke } from "@tauri-apps/api/core";
import type { EditorialRound, EditorialSession } from "./editorial";

/** One writer per review; each committed generation is based on the last ack.
 * A synchronous recovery journal covers the debounce and interrupted IPC window. */
export class EditorialSaves {
  private expected: number | null;
  private generationFloor: number;
  private pending: EditorialSession | null = null;
  private running: Promise<void> | null = null;
  readonly key: string;

  constructor(
    readonly round: EditorialRound,
    saved: EditorialSession | null,
    savedGeneration?: number | null
  ) {
    this.expected = savedGeneration === undefined ? (saved?.generation ?? null) : savedGeneration;
    this.generationFloor = saved?.generation ?? 0;
    this.key = `kindling.editorial.recovery.${round.id}`;
  }

  recover(saved: EditorialSession | null): EditorialSession | null {
    const text = localStorage.getItem(this.key);
    if (!text) return saved;
    const draft = JSON.parse(text) as { round: EditorialRound; session: EditorialSession };
    if (JSON.stringify(draft.round) !== JSON.stringify(this.round))
      throw new Error(
        "The recovered review has a different manuscript. Keep this window open and export a recovery copy."
      );
    if (saved && saved.reviewer_id !== draft.session.reviewer_id) return saved;
    const selected = saved && saved.generation > draft.session.generation ? saved : draft.session;
    const other = selected === saved ? draft.session : saved;
    // A writer response may have added replies to the saved session while an
    // interrupted local edit still exists in the journal. Preserve both.
    for (const incoming of other?.changes ?? []) {
      const change = selected.changes.find((c) => c.id === incoming.id);
      if (change) {
        for (const note of incoming.messages)
          if (!change.messages.some((m) => m.id === note.id)) change.messages.push(note);
        if (
          change.revision === incoming.revision &&
          (other?.writer_version ?? 0) >= (selected.writer_version ?? 0) &&
          incoming.writer_decision
        )
          change.writer_decision = incoming.writer_decision;
      } else if (incoming.messages.length) {
        const discussion = structuredClone(incoming);
        if (discussion.kind !== "comment") discussion.id = `discussion-${discussion.id}`;
        discussion.kind = "comment";
        discussion.state = "open";
        discussion.writer_decision = undefined;
        discussion.after = null;
        discussion.anchor_offset = null;
        const existing = selected.changes.find((c) => c.id === discussion.id);
        if (existing) {
          for (const note of discussion.messages)
            if (!existing.messages.some((m) => m.id === note.id)) existing.messages.push(note);
        } else selected.changes.push(discussion);
      }
    }
    selected.writer_version = Math.max(other?.writer_version ?? 0, selected.writer_version ?? 0);
    this.pending = selected;
    return selected;
  }

  recoveryGeneration() {
    return Math.max(this.expected ?? 0, this.generationFloor, this.pending?.generation ?? 0) + 1;
  }

  stage(session: EditorialSession) {
    this.pending = {
      ...structuredClone(session),
      generation: Math.max(this.expected ?? 0, this.generationFloor) + 1,
    };
    localStorage.setItem(this.key, JSON.stringify({ round: this.round, session: this.pending }));
  }

  discard() {
    localStorage.removeItem(this.key);
    this.pending = null;
  }

  async flush(): Promise<number | null> {
    if (this.running) {
      await this.running;
      return this.flush();
    }
    this.running = this.write();
    try {
      await this.running;
      return this.expected;
    } finally {
      this.running = null;
    }
  }

  private async write() {
    while (this.pending) {
      const draft = this.pending;
      const session = {
        ...draft,
        generation: Math.max(this.expected ?? 0, this.generationFloor) + 1,
      };
      await invoke("save_editorial_session", {
        round: this.round,
        session,
        expectedGeneration: this.expected,
      });
      this.expected = session.generation;
      if (this.pending === draft) {
        this.pending = null;
        localStorage.removeItem(this.key);
      } else if (this.pending) {
        this.stage(this.pending);
      }
    }
  }
}
