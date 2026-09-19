# 15 Find and replace

Socket: `npm run qa:visual -- --only 15`. This captures populated project
results, no results, replace-all confirmation and undo in the standard matrix.
The fixture prose contains two occurrences of “lantern”; replacement uses “beacon”.
Inspect the result path, highlighted context, scope/flags, locked/draft indicators,
confirmation and status. Search results and controls must fit at 1100×700.

Additional acceptance (record separately):

1. Open Find in Scene and assert only the selected scene contributes matches.
   Toggle Match case and Whole words using mixed-case and partial-word fixtures.
2. Step Next/Previous and Shift+Enter/Enter. Open scene from a match in a collapsed
   chapter/beat; verify the matching beat expands and the correct prose is selected.
3. Replace a single match, replace all, then undo both while the dialog stays open.
   Read saved prose through IPC to verify original formatting outside the match.
4. Lock a scene and a chapter in an owned fixture. Results remain searchable;
   replacements skip them and the confirmation count explains the exclusions.
5. Exercise retained unsaved drafts/retry/discard using component fault injection.
   A failed save must not allow replacement to silently overwrite a draft.
6. Repeat on Page prose, archived/flexible/undefined scenes, and no matches.
   Compare expected inclusions with the scope description in the dialog.
