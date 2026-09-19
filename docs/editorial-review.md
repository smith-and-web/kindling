# Editorial review without accounts

Kindling can send a manuscript to an editor and bring their feedback back as
local files. Both people use Kindling. The review needs no account, server, or
network connection; choose your own way to transfer the files.

## Send a manuscript

Open your project and choose **File → Editorial Review** (also available in the
command palette). Name the review round and add an optional brief. **Entire
manuscript** is selected by default. To send part of the manuscript, choose
**Selected chapters** and check at least one chapter. Past rounds and the action
for opening returned feedback appear beside the package form.

When you open package setup from revisions, Back returns to the same review or
suggesting mode and manuscript position. Review rounds show their creation date
and local time, including seconds, so rounds created on the same day are distinct.

**Export review package** creates a `.kindling-review` file. It contains the
active prose in manuscript order, including scenes written using beats. Your
original manuscript version is retained locally for this review round. You can
keep writing while your editor reviews that version.

Each export starts a new round. Use a new round when you want an editor to read
your updated manuscript, rather than continuing their earlier review.

## Review the manuscript

Double-click the package or choose **Open a review or feedback package** on the
start screen. **File → Open Review or Feedback File** works inside a project too.
Opening a package preserves any writing project you already have open.

Enter the name you want shown with your feedback once. Kindling remembers it on
this installation; change it under **Review options → Your name**. It is an
attribution label, not an account or verified identity.
An empty name is highlighted with a reminder; enter a name to add comments and
export feedback. The manuscript navigator groups scenes under their chapters
and highlights the current scene.

Packages and in-project revisions use the same manuscript, search, menus, and
feedback sidebar. Opening a package adds manuscript navigation so an editor can
work without the writer's project.

Read and edit the manuscript normally. Typing, deleting, replacing selections,
pasting, and formatting create suggestions. Use the formatting toolbar and
normal undo/redo shortcuts. Selecting across beats or scenes is one editorial
action. Chapter and scene records remain intact: a replacement belongs to the
first surviving source, and consumed prose is removed from subsequent sources.

Click **Comment**, or use the default **Cmd/Ctrl+Alt+M** shortcut, to comment on a selection. With a
cursor and no selected text, the comment records broader feedback at that
reading position. Threads support replies, resolution, and reopening. Refine
your suggestions by editing them in the manuscript, or select **Withdraw**.

Use **Find in manuscript** or the default **Cmd/Ctrl+F** shortcut, chapter navigation, and previous/next
annotation controls to move through the work. **Simple markup** keeps the prose
readable and shows paragraph-margin indicators. Click an indicator or a sidebar
thread to inspect that change beside its passage. Choose **All markup** to show
every pending insertion and deletion. The suggested text remains editable.

Work saves locally, with a recovery journal for interrupted saves. Reopening
the same review resumes its work and reading position. Watch the save indicator;
if saving fails, keep the workspace open and retry or export
a recovery review. Open the `.kindling-review` recovery file on this or another
installation to resume, then export feedback normally. A recovery copy can be
created even before entering your display name.

## Return and decide on feedback

**Manuscript actions (⋯) → Export feedback** creates a `.kindling-feedback` file. Return it to the writer.
You can send a partial review and continue working, then export another response.

The writer opens the response to see who returned it, the scenes included,
suggestion and comment counts, and the original review round and brief. Choose
**Import and review feedback** to read it alongside the manuscript. Import
adds annotations; prose changes only when the writer accepts suggestions. Open
previously imported feedback under **File → Editorial Review → Review rounds**.

Inspect suggestions individually or use the explicitly scoped accept/reject
actions under **Review options (⋯)**. The menu names whether the action applies
to the current round or saved scene suggestions. Formatting is preserved
and shown in the comparison. Accepting changes saves the previous scene prose in
**Revisions → Manuscript actions (⋯) → Draft history**. Locked scenes must be unlocked first.

If a passage has changed since export, compare the original passage, current
prose, and suggestion. Select the intended passage in the current manuscript and
choose **Apply to selected passage**. Overlapping suggestions must be reviewed
individually; a failed bulk acceptance does not partially apply prose changes.
If scenes or beats have been removed or switched to another editing mode,
feedback on changed passages needs a new selection in the current manuscript.
Suggestions on unrelated intact passages can still be accepted.

Repeated imports do not duplicate the same feedback or overwrite writer
decisions. Updated suggestions remain distinguishable from their earlier
versions. Responses from separate reviewers and separate rounds retain their
own identities. A response must return to the original project and registered
review round; matching project titles alone are insufficient.

Writers can use **Review options (⋯) → Export replies and decisions**
to send a `.kindling-review` response to a selected editor. Opening it merges
replies and shows the writer's decisions while preserving newer local review work.
Replies to earlier suggestion versions remain available as discussions. Start a
new round when both people need to review an updated manuscript baseline.

Scene locks protect the writer's stored prose. Editors can suggest changes to
exported locked scenes, but the writer must unlock those scenes before accepting
them. Local review also protects locked scenes from new suggestions and comments.

## Review inside your writing project

Choose **Revisions** above the current scene to open review in the main editor.
Your outline stays on the left, the manuscript fills the writing area, and the
right sidebar switches between **Review** and **References**. Existing scene
comments, suggestions, and draft history remain available.

Use the mode selector to switch between **Reviewing** (read current prose and
decide on feedback), **Suggesting** (type proposed edits), and **Writing** (return
to your writing view). Kindling preserves the selected passage between review
modes. Suggestions save locally and appear for decisions without exporting files.
The local pass retains its original manuscript so ongoing suggestions can resume;
if you have rewritten the manuscript, choose **Manuscript actions (⋯) → Start
suggestions from current prose** to begin a new pass. Earlier passes remain under
**Review packages and rounds**.

Click a thread to reply, resolve a comment, or accept/reject an edit. Conflicts
show the original, current, and suggested passage beside the text. Saved scene
suggestions are decided in Reviewing mode. Their older anchors still belong to
their original scene; new suggestions support continuous selections across scenes.
Annotations on inactive beat/page prose remain readable in the sidebar with
their original discussion. Their notice explains which scene and editing mode
to return to before acting on them. Bulk scene decisions apply to active prose.

**Manuscript actions (⋯) → Draft history** opens saved versions and comparisons.
Choose a saved draft from the list, then use **Compare with** to choose current
prose or another draft. The two versions appear side by side: removed text on
the left, added text on the right. Only each version's active prose is compared;
formatting changes are not included. Restoring a draft preserves the current
prose as another saved draft before replacing it.

Search highlights the active match and scrolls the manuscript to it, while the
search field keeps focus. Its controls stay above the scrolling manuscript, with
the formatting toolbar directly below. Previous/Next and Enter/Shift+Enter move through
matches. Selecting a sidebar thread also brings its passage into view.
Refreshing the manuscript or selecting a past round retains the search and
updates its matches. Opening a review file or starting local revisions clears
the previous search.
Package setup, past rounds, and recovery exports also live under Manuscript actions.
The revision status beside the scene title reflects the scene at the cursor.

## File opening and compatibility

Installed desktop builds register the two editorial file extensions. Registration
and default-app selection depend on the operating system and installation format;
the in-app Open action is always available. Development builds do not install
system file associations.

Packages include a format version. Unsupported versions, mismatched rounds, and
invalid annotations are reported before review. The current file size limit is
64 MiB; export fewer chapters if a package exceeds it.

## Explore the sample

Create a new **Sample Project** to explore the revision examples in _The Letter_:

| Scene                | Examples                                                                                                                  |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| On the Cliff         | Editor Review; named draft differences; writer/editor replies; insertion, deletion, replacement, and disjoint suggestions |
| The Seventh Step     | Overlapping alternatives; outdated anchor; comparable draft with incompatible beat structure                              |
| Supper with Silas    | Revised; accepted/rejected suggestions, resolved thread, preserved earlier prose                                          |
| Low Tide             | First Draft; no feedback or saved drafts                                                                                  |
| The Damaged Register | Page-mode review; reopened discussion; annotation on inactive beat prose                                                  |
| Margaret's Room      | Final; completed history; locked, read-only review                                                                        |

Revision status and the scene's drafting status stay aligned: Editor Review is
the review-specific stage within Draft. Changing a scene to Revised or Final
also updates its revision status, and accepted suggestions set it to Revised.
Moving a Revised or Final scene back to Draft starts it at First Draft; choose
Editor Review in Revisions when it is ready for that stage again.

Existing sample projects are not overwritten when Kindling updates. Create a
fresh sample for these examples; keep any sample you have edited separately.
