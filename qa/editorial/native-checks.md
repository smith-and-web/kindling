# Native editorial package checks

Use disposable app data and manuscript fixtures. Do not overwrite a writer's
existing sample or register a test bundle under their production app identifier.

Before investigating differences between in-app and file-manager opening,
resolve the actual file handler and check that executable's build time. An old
QA bundle can remain the default even while current source is running in dev.
Ordinary UI checks should open files explicitly with the current test executable
and isolated data, without registering another default application.

Association checks must record the previous handlers. Immediately afterward,
unregister temporary QA bundles and restore the intended current application for
both `.kindling-review` and `.kindling-feedback`. Verify resolution using actual
files of both types. Keep any open review session and its data intact while
repairing registration; do not terminate a user's review to clean up a handler.

1. Install a build without associations, then update it to the candidate build.
2. Check both file extensions' type name, document icon and Open With identity.
3. With Kindling closed, open a review file from the file manager. It must enter
   Editorial directly, including on a profile without projects or onboarding history.
4. Edit, comment, scroll without moving the cursor, close and reopen. Verify the
   same session, comments, suggestions and reading position resume.
5. Open a second file while Kindling is running with unsaved writer work. Verify
   writing saves and the underlying project remains open when Editorial closes.
6. Export partial feedback; continue both editor and writer work; return again.
   Import must preserve current writer prose and previous decisions.
7. Accept/reject, reanchor a conflict and bulk-accept disjoint changes. Confirm
   previous scene prose is in draft history and locks block the complete action.
8. Export writer replies/decisions. Open that response on the editor profile with
   newer local work; verify both that work and the received discussion survive.
9. Export a recovery review during a simulated save failure. Open it on a fresh
   editor profile, continue, return feedback and import it into the writer profile.
10. Repeat import/open; try an unsupported version and a different project identity.
    Check the error and confirm no prose or existing review is replaced.

Run this against macOS app/DMG replacement, Windows MSI and NSIS installation
and updater paths, and the supported Linux deb/RPM/AppImage distribution paths.
An AppImage may require desktop integration to register its file types; verify
that integration separately from launching the image directly. Always check the
in-app Open action as well.

Tauri's association configuration and exported macOS document identities are
specified in its [configuration reference](https://v2.tauri.app/reference/config/).
A configuration/source review alone does not establish installer behavior.
