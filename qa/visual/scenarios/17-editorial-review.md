# 17 Editorial review and portable packages

Socket: `npm run qa:visual -- --only 17`. Captures the local editorial workspace,
manuscript actions menu, Draft history, All scenes and review package setup with
real fixture prose in light/dark/narrow. This is surface coverage; it does not
export or accept a portable review.

Expect the manuscript and feedback sidebar to share the available width, readable
insertions/deletions and focus states, menus within the viewport, and a usable
comparison/history pane. Package setup must leave scope and export actions visible.

Complete [editorial acceptance](../../editorial/acceptance.md) and
[native checks](../../editorial/native-checks.md), recording these additional
screenshots in each theme plus narrow:

1. Enter Suggesting; type/delete/paste/format and add a selection comment. Capture
   Simple markup and All markup with both insertions and deletions present.
2. Reply, resolve, reopen and filter comments. Navigate annotations, search the
   manuscript and switch scenes without losing the selection or pending feedback.
3. Save a named draft, change prose, capture comparison, then restore in the owned
   project. Check prose/status and writing-credit behavior through IPC.
4. Export a `.kindling-review` into the run folder, open it as editor, make and save
   suggestions/comments, export feedback, then open that feedback as the writer.
   Capture the feedback summary and accept/reject preview; verify selective decisions.
5. Keep writing while a round is out. Start a second round, return older feedback,
   and verify snapshot association, conflicts and already-applied decisions.
6. Exercise offline save failure/recovery, pending changes when switching/closing,
   and portable file reopen after a native restart. Record actual file paths and
   returned IDs; never overwrite a personal manuscript or review package.
