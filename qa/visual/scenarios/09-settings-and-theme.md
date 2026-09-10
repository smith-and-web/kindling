# 09 Unified settings and themes

Use a disposable project database and save/restore `kindling:*` preferences with the harness.

1. Open **File → Settings** (or send `q.key(",", { metaKey: true })`). Verify a single native dialog titled Settings, a left navigation sidebar, and right-hand controls. The sidebar, References panel, and start screen should have no separate settings gears.
2. Select **Author & Contact**. Confirm loaded values and edit a draft. Navigate to Appearance and back; the draft should survive. Close Settings, choose Keep editing, then close and discard. Verify reopening loads the saved values.
3. Select **Project Details**. Verify the selector lists all fixture projects and defaults to the open project. Change the word target and daily goal; save, close, reopen, and verify persistence.
4. Switch to a second fixture project. Edit and save its genre. Verify the editor's project and scene stay unchanged, and that the first project's metadata has not changed.
5. Edit a draft, switch projects, and exercise both Keep editing and Discard changes. Repeat with a tag draft and a custom-field draft. Navigate between areas to verify drafts survive.
6. Select **Reference Types**, change enabled types, and save. Confirm the selected project's References panel refreshes; disabled types retain their existing entries.
7. Select **Tags** and **Custom Fields** and exercise create/edit/delete using disposable entries. Check the target project IDs and that the editor sees updates for the open project.
8. Select **Appearance & Guidance**, click the Dark radio, and capture `09-01-unified-settings-dark`. Inspect computed dialog/sidebar/control colors. Click Light, inspect again, and capture `09-02-unified-settings-light`.
9. Capture Project Details at 1600×1000 and at a narrow viewport. Verify the left navigation stays visible, controls scroll vertically, and there is no horizontal overflow.
10. Close the project and reopen Settings from File. Project navigation and shared preferences must remain available. With no projects, show an empty state without disabling shared settings.
11. Restore preferences and delete only the fixtures created during this run.

Automated regression tests cover rejected list/author loads, retries, rejected saves, and navigation guards during writes. Record desktop IPC checks and screenshot observations separately in the acceptance report.
