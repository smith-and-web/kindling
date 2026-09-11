# 16 Writing statistics and previous scene context

Socket: `npm run qa:visual -- --only 16`. Seed prose only in owned fixture scenes;
assert no Previously on the first scene, then capture expanded/collapsed context,
statistics and the collapsed-outline layout. Every state runs light/dark/narrow.

Expect the Previously/Revisions row to align, the excerpt to use Newsreader and
bounded prose width, and the statistics table and status totals to remain readable
with either sidebar state. No horizontal document overflow is acceptable.

Additional acceptance:

1. Edit and save Beat and Page prose through the editor. Verify scene/chapter/
   project/session counts after save, including net negative edits and session reset.
2. Set a daily goal in Project Details, persist/reopen, then set zero. Verify goal
   visibility and progress. Test midnight and unfinished-today streak rules with
   controlled-clock tests; do not change the workstation clock.
3. Import, duplicate, reorder, restore a draft and accept editorial suggestions.
   Verify these do not earn writing credit. IPC seeding in the socket sweep does
   not prove this rule or the editor save path.
4. Verify Previously crosses chapter boundaries in manuscript order, skips archived
   content, shows at most the last three sentences, uses the active Beat/Page prose,
   and never substitutes outline prompts for prose.
5. Collapse, navigate away/back, reload and reopen the same owned project: the
   collapse preference persists. Exercise the read-failure retry in component tests.
6. Compare actual saved counts with expected fixture totals, including empty scenes;
   a visually correct table with stale values is still a regression.
