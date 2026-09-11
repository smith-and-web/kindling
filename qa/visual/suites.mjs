// Executable checkpoints. The numbered Markdown scenarios describe deeper/manual
// acceptance checks; this file is the source of truth for automated coverage.
const el = (css) => `document.querySelector(${JSON.stringify(css)})`;
const settings = '[data-testid="settings-dialog"]';
const find = 'dialog[aria-labelledby="find-title"]';

export const suites = [
  {
    id: "23",
    title: "Remembered custom export profiles",
    async run(r) {
      await r.seedProse();
      await r.scene();
      await r.command("export");
      await r.wait(el('[data-testid="export-format-custom"]'));
      await r.check(`${el('input[name="format"][value="docx"]')}.checked`, "New projects keep the standard Word default");
      await r.js(`document.querySelector('[data-testid="export-format-custom"]').click();`);
      await r.wait(el('#custom-export-profile'));
      await r.fill('#custom-export-profile', 'website');
      await r.click('[data-testid="export-close"]');
      await r.command('export');
      await r.wait(`${el('[data-testid="export-format-custom"]')}?.checked && ${el('#custom-export-profile')}?.value === 'website'`);
      await r.check(`!document.querySelector('#epub-title')`, 'Custom profiles must not show standard EPUB settings');
      await r.shot('23-01-custom-profile-default', 'Custom profiles is a selected format tile, with the remembered profile and an Open workspace action.');
      await r.click('[data-testid="export-confirm"]');
      await r.wait(`${el('#export-profile')}?.value === 'website'`);
      await r.fill('#export-profile', 'readers');
      await r.wait(`${el('#export-format')}?.value === 'epub'`);
      await r.text('Back to export');
      await r.wait(`${el('#custom-export-profile')}?.value === 'readers'`);
      // A new selection in the standard dialog must win over a different workspace draft.
      await r.fill('#custom-export-profile', 'website');
      await r.click('[data-testid="export-confirm"]');
      await r.wait(`${el('#export-profile')}?.value === 'website'`);
      await r.click('[aria-label="Duplicate profile"]');
      await r.fill('#export-name', 'My submission profile');
      await r.text('Save profile');
      const profileId = await r.js(`return document.querySelector('#export-profile').value;`);
      await r.click('[aria-label="Close export workspace"]');
      await r.command('export');
      await r.wait(`${el('#custom-export-profile')}?.value === ${JSON.stringify(profileId)}`);
      await r.check(`${el('#custom-export-profile')}.selectedOptions[0].textContent === 'My submission profile'`);
      await r.shot('23-02-custom-profile-restored', 'A profile created in the workspace is remembered in the standard export dialog.');
      // Choosing a standard format for one export does not discard the custom default.
      await r.js(`document.querySelector('input[name="format"][value="docx"]').click();`);
      await r.click('[data-testid="export-close"]');
      await r.command('export');
      await r.wait(`${el('[data-testid="export-format-custom"]')}?.checked && ${el('#custom-export-profile')}?.value === ${JSON.stringify(profileId)}`);
      await r.click('[data-testid="export-close"]');
      await r.js(`localStorage.removeItem(${JSON.stringify(`kindling:export-prototype:v1:${r.project.id}`)}); localStorage.removeItem(${JSON.stringify(`kindling:custom-export:${r.project.id}`)});`);
    },
  },
  {
    id: "00",
    title: "Start screen and project creation dialog",
    fixture: false,
    async run(r) {
      await r.shot(
        "00-03-start-screen",
        "Aligned project/review entry points and empty-library placeholder; six import formats are available in the scrollable start screen.",
        `${el('[data-testid="import-section"]')}.textContent.includes('novelWriter') && document.body.textContent.includes('review')`
      );
      await r.click('[data-testid="new-project-button"]');
      await r.wait(el("#new-project-dialog-title"));
      await r.shot(
        "00-05-new-project",
        "Novel/screenplay choices, template selection and project name fit the dialog.",
        el('[data-testid="new-project-create"]')
      );
      await r.click('[data-testid="new-project-close"]');
    },
  },
  {
    id: "01",
    title: "Outline creation and editor shell",
    async run(r) {
      await r.scene();
      await r.shot(
        "01-09-scene-selected",
        "Outline, synopsis, scene tools, beats, references and writing status are all readable.",
        el('[data-testid="beat-header"]')
      );
      await r.click('[data-testid="new-chapter-button"]');
      await r.wait(el('[data-testid="title-input"]'));
      await r.shot(
        "01-01-chapter-input",
        "Inline new chapter input aligns with the outline and has visible focus."
      );
      await r.js('q.fillTitle("QA Chapter");');
      await r.wait(
        `[...document.querySelectorAll('[data-testid="chapter-title"]')].some(e=>e.textContent.trim()==='QA Chapter')`
      );
      await r.shot(
        "01-02-chapter-created",
        "New chapter appears in manuscript order, selected with empty-scene creation actions."
      );
    },
  },
  {
    id: "07",
    title: "Beat and Page prose",
    async run(r) {
      await r.seedProse();
      await r.scene();
      await r.click('[data-testid="beat-header"]');
      await r.wait(el('[data-testid="beat-prose-editor"]'));
      // BeatView schedules an explicit native smooth scroll after its Svelte
      // tick. Let that finish before positioning the comparison viewport.
      await new Promise((resolve) => setTimeout(resolve, 1000));
      await r.js(
        "document.querySelector('[data-testid=beat-prose-editor]').closest('.novel-editor').scrollIntoView({block:'start',behavior:'instant'});"
      );
      await r.shot(
        "07-00-beat-prose",
        "Newsreader prose, formatting toolbar, beat prompt and word count fit the writing column.",
        `${el('[data-testid="beat-prose-editor"]')}.textContent.includes('lantern') &&
          (()=>{const p=${el('[data-testid="beat-prose-editor"] p')}.getBoundingClientRect();return p.top>=0 && p.bottom<innerHeight;})()`
      );
      await r.click('[data-testid="view-page"]');
      await r.wait(
        "document.querySelector('.novel-editor-content') && !document.querySelector('[data-testid=\"beat-header\"]')"
      );
      await r.js(
        "document.querySelector('.novel-editor-content').closest('.novel-editor').scrollIntoView({block:'start',behavior:'instant'});"
      );
      await r.shot(
        "07-01-page-view",
        "Page writing surface and toolbar use the correct theme and prose measure."
      );
      await r.click('[data-testid="view-beats"]');
      // If Page view has no independent prose, switching back may need no confirmation.
      if (await r.js("return !!document.querySelector('[data-testid=\"confirm-dialog\"]');")) {
        await r.shot(
          "07-02-switch-confirm",
          "Switch-back warning clearly describes what happens to Page prose."
        );
        await r.dismiss();
      }
    },
  },
  {
    id: "08",
    title: "References and copy preview",
    async run(r) {
      const source = r.project;
      await r.scene();
      await r.click('[data-testid="add-reference-button"]');
      await r.wait(el('[aria-labelledby="reference-dialog-title"]'));
      await r.shot(
        "08-01-reference-dialog",
        "Name, description, notes and attributes are readable; Save is disabled for an empty name.",
        `${el('[data-testid="reference-save"]')}.disabled`
      );
      await r.fill('[placeholder="Enter name..."]', "QA Mara");
      await r.click('[data-testid="reference-save"]');
      await r.wait(
        "!document.querySelector('[aria-labelledby=\"reference-dialog-title\"]') && document.body.textContent.includes('QA Mara')"
      );
      await r.shot(
        "08-02-reference-listed",
        "Created character is listed with edit/link controls and an updated category count."
      );
      await r.command("close_project");
      await r.wait(el('[data-testid="import-section"]'));
      await r.fixture();
      await r.scene();
      await r.click('[aria-label="Copy references from project…"]');
      await r.wait(`${el("#copy-source")} && !${el("#copy-source")}.disabled`);
      await r.fill("#copy-source", source.id);
      await r.wait(
        "document.querySelector('#copy-search') && document.querySelector('dialog[open]').textContent.includes('QA Mara')"
      );
      await r.shot(
        "08-03-copy-references",
        "Source selector, category selection, search and copy summary fit within the scrolling dialog.",
        `![...document.querySelector('#copy-source').options].some(o=>o.value===${JSON.stringify(r.project.id)})`
      );
      await r.fill("#copy-search", "not present");
      await r.shot(
        "08-04-copy-filter-empty",
        "An empty filtered library leaves source selection and footer actions reachable."
      );
      await r.dismiss();
    },
  },
  {
    id: "09",
    title: "Unified settings areas and project scope",
    async run(r) {
      await r.scene();
      await r.click('[data-testid="sidebar-settings-button"]');
      await r.wait(el(settings));
      await r.wait("document.querySelector('#settings-project')?.options.length > 0");
      const areas = [
        "Appearance & Guidance",
        "Author & Contact",
        "Project Details",
        "Reference Types",
        "Tags",
        "Custom Fields",
      ];
      for (const [index, area] of areas.entries()) {
        await r.text(area, `${el(settings)}.querySelector('nav')`);
        await r.wait(
          `${el(settings)}.querySelector('button[aria-current="page"]')?.textContent.trim() === ${JSON.stringify(area)}`
        );
        if (area === "Appearance & Guidance") {
          await r.click(`[data-testid="theme-option-${r.variant === "dark" ? "dark" : "light"}"]`);
        }
        const authorFields = ["author-name", "address-line1", "address-line2", "phone", "email"];
        let originalAuthor;
        if (area === "Author & Contact") {
          await r.wait(
            "document.querySelector('#author-name') && !document.querySelector('#author-name').disabled"
          );
          originalAuthor = await r.js(
            `return ${JSON.stringify(authorFields)}.map(id=>document.getElementById(id).value);`
          );
        }
        try {
          if (originalAuthor) {
            const samples = [
              "QA Author",
              "123 Example Street",
              "Example City",
              "555-0100",
              "qa@example.test",
            ];
            for (const [i, field] of authorFields.entries()) await r.fill(`#${field}`, samples[i]);
            await r.js("document.activeElement?.blur();");
          }
          await r.shot(
            `09-0${index + 1}-${area.toLowerCase().replaceAll(/[^a-z]+/g, "-")}`,
            `${area}: left navigation and project selector stay visible; right-hand controls scroll without horizontal overflow.`,
            `${el("#settings-project")}.value === ${JSON.stringify(r.project.id)} && ${el('[data-testid="settings-location"]')}`
          );
        } finally {
          if (originalAuthor) {
            for (const [i, field] of authorFields.entries())
              await r.fill(`#${field}`, originalAuthor[i]);
            await r.js("document.activeElement?.blur();");
          }
        }
      }
      await r.click('[data-testid="settings-close"]');
      await r.wait(`!${el(settings)}`);
      await r.check(
        `${el('[data-testid="scene-panel"] [data-testid="scene-title"]')}.textContent.trim() === 'The Beginning'`,
        "Settings changed the selected scene"
      );
    },
  },
  {
    id: "10",
    title: "Export, snapshots and command palette",
    async run(r) {
      await r.scene();
      await r.command("export");
      await r.wait(el("#export-dialog-title"));
      await r.shot(
        "10-01-export-dialog",
        "Export format choices, scope and output controls are readable, including novelWriter options."
      );
      await r.click('[data-testid="export-close"]');
      await r.click('[data-testid="more-actions-button"]');
      await r.wait(el('[data-testid="snapshots-button"]'));
      await r.click('[data-testid="snapshots-button"]');
      await r.wait(el("#snapshots-panel-title"));
      await r.shot(
        "10-02-snapshots-empty",
        "Snapshots empty state and Create Snapshot action are visible.",
        "document.body.textContent.includes('No snapshots yet')"
      );
      await r.click('[data-testid="snapshot-create-button"]');
      await r.wait(el("#snapshot-name"));
      await r.fill("#snapshot-name", "QA snapshot");
      await r.click('[data-testid="snapshot-confirm-create"]');
      await r.wait(
        "!document.querySelector('#snapshot-name') && document.body.textContent.includes('QA snapshot')"
      );
      await r.shot(
        "10-03-snapshot-created",
        "Snapshot name, date, restore and delete controls align in the panel."
      );
      await r.click('[data-testid="snapshots-close"]');
      await r.command("command_palette");
      await r.wait(el('[aria-label="Command palette"]'));
      await r.shot(
        "10-04-command-palette",
        "Search input and commands show readable current shortcut hints."
      );
      await r.dismiss();
    },
  },
  {
    id: "15",
    title: "Find, replace, no matches and undo",
    async run(r) {
      await r.seedProse();
      await r.scene();
      await r.command("find_project");
      await r.wait(el(find));
      await r.wait(`!${el(find)}.querySelector('fieldset').disabled`);
      await r.check(`${el(find)}.querySelector('header button[aria-label="Close Find and Replace"] svg')`, "Find uses the shared header and X close button");
      await r.fill(`${find} input[type="text"]`, "lantern");
      await r.wait(`${el(find)}.querySelector('mark')`);
      await r.js(`${el(find)}.querySelector('mark').scrollIntoView({block:'center'});`);
      await r.shot(
        "15-01-project-results",
        "Result path, highlighted match, scope and replacement actions remain readable.",
        `${el(find)}.querySelector('select').value === 'project'`
      );
      await r.fill(`${find} input[type="text"]`, "NoSuchQAText");
      await r.wait(`${el(find)}.textContent.includes('No matches found.')`);
      await r.shot(
        "15-02-no-results",
        "No-match message is legible and replacement buttons are disabled."
      );
      await r.fill(`${find} input[type="text"]`, "lantern");
      await r.js(`const input=[...${el(find)}.querySelectorAll('input[type=text]')][1];
        if(!input) throw new Error('Missing replacement input');input.value='beacon'; input.dispatchEvent(new Event('input',{bubbles:true}));`);
      await r.text("Replace all", el(find));
      await r.wait(`${el(find)}.textContent.includes('Confirm replace all')`);
      await r.js(
        `[...${el(find)}.querySelectorAll('button')].find(b=>b.textContent.trim()==='Confirm replace all').scrollIntoView({block:'end'});`
      );
      await r.shot(
        "15-03-replace-confirm",
        "Confirmation names scope and count, explains skipped locked/draft matches and provides Cancel."
      );
      await r.text("Confirm replace all", el(find));
      await r.wait(
        `!${el(find)}.querySelector('fieldset').disabled && [...${el(find)}.querySelectorAll('button')].some(b=>b.textContent.trim()==='Undo replacement' && !b.disabled)`
      );
      await r.text("Undo replacement", el(find));
      await r.wait(`${el(find)}.querySelector('mark')?.textContent === 'lantern'`);
      await r.js(
        `[...${el(find)}.querySelectorAll('p')].find(e=>e.textContent.trim()==='Replacement undone.')?.scrollIntoView({block:'end'});`
      );
      await r.shot(
        "15-04-undo",
        "Undo restores highlighted original prose and leaves the dialog usable."
      );
      await r.dismiss();
    },
  },
  {
    id: "16",
    title: "Writing statistics and Previously",
    async run(r) {
      await r.seedProse();
      await r.scene();
      await r.check(
        "!document.querySelector('[aria-controls=\"previously-content\"]')",
        "First scene unexpectedly has a predecessor"
      );
      await r.scene("Discovery");
      await r.wait(el('[aria-controls="previously-content"]'));
      if (!(await r.js(`return !!${el("#previously-content")};`)))
        await r.click('[aria-controls="previously-content"]');
      await r.wait(`${el("#previously-content")}?.textContent.includes('letter home')`);
      await r.shot(
        "16-01-previously",
        "Previously and Revisions share a scene-tools row; predecessor title, synopsis and excerpt use readable prose measure."
      );
      await r.click('[aria-controls="previously-content"]');
      await r.wait(`!${el("#previously-content")}`);
      await r.shot(
        "16-02-previously-collapsed",
        "Collapsed context leaves scene tools aligned and avoids a large blank gap."
      );
      await r.click('[aria-controls="writing-statistics-panel"]');
      await r.wait(el("#writing-statistics-panel"));
      await r.shot(
        "16-03-writing-statistics",
        "Scene/chapter/project/session totals, chapter breakdown and empty/prose scene counts fit the panel.",
        `${el("#writing-statistics-panel")}.textContent.includes('Words per chapter')`
      );
      await r.command("toggle_sidebar");
      await r.shot(
        "16-04-collapsed-sidebar-statistics",
        "Writing statistics and status totals remain usable with the outline collapsed."
      );
      await r.command("toggle_sidebar");
      await r.click('[aria-controls="writing-statistics-panel"]');
    },
  },
  {
    id: "17",
    title: "Editorial review, manuscript menus and draft history",
    async run(r) {
      await r.seedProse();
      await r.scene();
      await r.text("Revisions");
      await r.wait(el('[aria-label="Editorial workspace"]:not([hidden])'));
      await r.wait(el('[aria-label="Manuscript"]'));
      await r.shot(
        "17-01-editorial-workspace",
        "Manuscript, revision status, mode controls and feedback sidebar fit together.",
        el('[aria-label="Editorial feedback"]')
      );
      await r.click('[aria-label="Manuscript actions"]');
      const menuFits = `(() => { const menu = document.querySelector('[data-testid="context-menu"]');
        if (!menu) return false; const b = menu.getBoundingClientRect();
        return b.width > 0 && b.left >= 0 && b.top >= 0 && b.right <= innerWidth && b.bottom <= innerHeight; })()`;
      await r.wait(menuFits);
      await r.shot(
        "17-02-manuscript-menu",
        "Manuscript menu is inside the viewport with readable actions and icons.",
        menuFits
      );
      await r.text("Draft history");
      await r.wait(el("#revisions-title"));
      await r.shot(
        "17-03-draft-history",
        "Saved draft list, comparison pane, status and All scenes navigation fit the dialog."
      );
      await r.text("All scenes", `${el("#revisions-title")}.closest('dialog')`);
      await r.shot(
        "17-04-all-scenes",
        "Revision overview lists manuscript scenes and statuses without clipping."
      );
      await r.dismiss();
      await r.click('[aria-label="Manuscript actions"]');
      await r.wait(el('[data-testid="context-menu"]'));
      await r.text("Review packages and rounds…");
      await r.wait(el("#package-title"));
      await r.shot(
        "17-05-review-package",
        "Review package setup shows the round name, editor brief, manuscript scope and round history in a balanced layout."
      );
      // A package opened from local revisions returns to the workspace before writing.
      await r.click('[aria-label="Return to revisions"]');
      await r.wait(el('[aria-label="Return to writing"]'));
      await r.click('[aria-label="Return to writing"]');
      await r.wait(`!${el('[aria-label="Editorial workspace"]:not([hidden])')}`);
    },
  },
  {
    id: "18",
    title: "About and feedback form (no submission)",
    async run(r) {
      await r.command("about");
      await r.wait(el('[data-testid="about-dialog"]'));
      await r.shot(
        "18-01-about",
        "About shows app version and the Send Feedback action with readable links."
      );
      await r.click('[data-testid="about-close"]');
      await r.command("send_feedback");
      await r.wait(el('[data-testid="feedback-dialog"]'));
      await r.shot(
        "18-02-feedback",
        "Feedback categories, optional summary, message and send action fit the form. No feedback is submitted.",
        el('[data-testid="feedback-submit"]')
      );
      await r.click('[data-testid="feedback-dialog"] [aria-label="Close"]');
    },
  },
  {
    id: "19",
    title: "Export workspace prototype",
    async run(r) {
      await r.seedProse();
      await r.scene();
      await r.command("export");
      await r.wait(el('[data-testid="export-format-custom"]'));
      await r.js(`document.querySelector('[data-testid="export-format-custom"]').click();`);
      await r.click('[data-testid="export-confirm"]');
      await r.wait(el('[data-testid="export-workspace"]'));
      await r.wait(`!document.querySelector('[aria-label="Refresh saved manuscript"]').disabled`);
      await r.check(`!document.querySelector('[data-testid="export-workspace"] [role="alert"]')`);
      await r.check(`!document.querySelector('.preview-tabs')`, "Word preview must not expose HTML source");
      await r.check(`!document.querySelector('[data-testid="export-workspace"]').textContent.includes('Prototype')`, "Workspace must not show prototype badges");
      await r.check(`(() => { const a=document.querySelector('#export-profile').getBoundingClientRect(); const b=document.querySelector('#export-format').getBoundingClientRect(); const c=document.querySelector('[aria-label="Duplicate profile"]').getBoundingClientRect(); return Math.abs(a.top-b.top)<1 && a.height===b.height && a.height===c.height; })()`, "Profile and format controls must align and share a height");
      await r.shot("19-01-export-overview", "Export profiles, configuration categories, manuscript preview and format options are readable.", `${el('iframe[title="Export layout preview"]')}.srcdoc.includes('lantern')`);
      await r.fill('[aria-label="Find an export setting"]', "double spaced");
      await r.text("Line spacing");
      await r.wait(el("#export-spacing"));
      await r.fill("#export-font", "Georgia");
      await r.fill("#export-spacing", "1.5");
      await r.shot("19-02-export-typography", "Typography and page controls sit beside a responsive layout approximation.", `${el('iframe[title="Export layout preview"]')}.srcdoc.includes('Georgia')`);
      await r.text("Save profile");
      await r.click('[aria-label="Close export workspace"]');
      await r.command("export");
      await r.wait(el('[data-testid="export-format-custom"]'));
      await r.js(`document.querySelector('[data-testid="export-format-custom"]').click();`);
      await r.click('[data-testid="export-confirm"]');
      await r.wait(el('[data-testid="export-workspace"]'));
      await r.fill('[aria-label="Find an export setting"]', "double spaced");
      await r.text("Line spacing");
      await r.check(`${el('#export-font')}.value === 'Georgia' && ${el('#export-spacing')}.value === '1.5'`, "Saved profile must survive closing and reopening");
      await r.fill("#export-format", "html");
      await r.fill('[aria-label="Find an export setting"]', "fragment");
      await r.text("HTML structure and styling");
      await r.fill("#export-html-mode", "fragment");
      await r.text("HTML", 'document.querySelector(".preview-tabs")');
      await r.shot("19-03-export-html", "HTML fragment settings, filename tokens and generated source are inspectable.", `${el('.source-preview')}.value.includes('<main') && !${el('.source-preview')}.value.includes('<!doctype')`);
      await r.fill('[aria-label="Find an export setting"]', "Included chapters");
      await r.text("Included chapters");
      await r.fill("#export-scope", "selected");
      await r.js(`document.querySelectorAll('.chapter-row input:checked').forEach(input => input.click());`);
      await r.wait(`document.querySelector('.empty-preview')`);
      await r.check(`[...document.querySelectorAll('button')].find(b=>b.textContent.includes('Export HTML')).disabled`, "Empty selection must disable preview export");
      await r.shot("19-04-export-selection", "Explicit chapter selection and its empty-preview state explain why export is unavailable.");
      await r.click('[aria-label="Close export workspace"]');
      await r.js(`localStorage.removeItem(${JSON.stringify(`kindling:export-prototype:v1:${r.project.id}`)}); localStorage.removeItem(${JSON.stringify(`kindling:custom-export:${r.project.id}`)});`);
    },
  },
  {
    id: "22",
    title: "Export workspace formats and generated files",
    async run(r) {
      await r.seedProse();
      await r.scene();
      await r.command("export");
      await r.wait(el('[data-testid="export-format-custom"]'));
      await r.js(`document.querySelector('[data-testid="export-format-custom"]').click();`);
      await r.click('[data-testid="export-confirm"]');
      await r.wait(`${el('[data-testid="export-workspace"]')} && !${el('[aria-label="Refresh saved manuscript"]')}.disabled`);
      await r.fill("#export-format", "epub");
      await r.check(`!document.querySelector('.preview-tabs')`, "EPUB preview must not expose HTML source");
      await r.js(`document.querySelectorAll('nav[aria-label="Export settings"] button').forEach(b => { if(b.textContent.includes('Book details')) b.click(); });`);
      await r.wait(el("#export-description"));
      await r.fill("#export-description", "A journey through the city & beyond.");
      await r.js(`document.querySelector('#export-description').scrollIntoView({block:'center'});`);
      await r.shot("22-01-export-epub", "Ebook metadata and optional cover controls accompany the manuscript preview.");
      for (const format of ["markdown", "txt"]) {
        await r.fill("#export-format", format);
        await r.wait(`${el('.source-preview')}?.value.includes('lantern')`);
        await r.check(`!${el('.source-preview')}.value.includes('<p>')`, "Text formats should show their actual output");
      }
      await r.fill("#export-format", "novelwriter");
      await r.js(`document.querySelectorAll('nav[aria-label="Export settings"] button').forEach(b => { if(b.textContent.includes('Files & format')) b.click(); });`);
      await r.wait(`document.querySelector('.settings-panel')?.textContent.includes('Reference notes')`);
      await r.shot("22-02-export-project", "Project export exposes notes and beat comments while explaining its whole-project scope.");
      await r.fill("#export-format", "treatment");
      await r.wait(el("#export-treatment-level"));
      await r.fill("#export-treatment-level", "full");
      await r.fill("#export-treatment-format", "txt");
      await r.shot("22-03-export-treatment", "Treatment detail and file type replace manuscript typography controls.");
      const chapters = await r.app.invoke("get_export_prototype_document", { projectId: r.project.id });
      await r.js(`import('/src/lib/utils/exportPrototype.ts').then(m => window.__workspaceExportModule=m);`);
      await r.wait("window.__workspaceExportModule");
      for (const format of ["docx", "epub", "markdown", "txt", "html"]) {
        const payload = await r.js(`const m=window.__workspaceExportModule; const p=m.starterProfiles('QA Export Book','QA Author')[0]; p.format=${JSON.stringify(format)}; p.contents=true; p.font='Georgia'; return {document:m.compileWorkspaceDocument(${JSON.stringify(chapters)},p),html:m.renderPreview(${JSON.stringify(chapters)},p).output};`);
        const path = `${r.output}/${r.variant}-workspace-${format}.${format === "markdown" ? "md" : format}`;
        if (format === "html") await r.app.invoke("save_export_prototype_html", { path, html: payload.html });
        else await r.app.invoke("export_workspace_document", { path, format, document: payload.document });
      }
      for (const format of ["longform", "scrivener", "novelwriter", "treatment"]) {
        const path = `${r.output}/${r.variant}-workspace-${format}${format === "scrivener" ? ".scriv" : format === "treatment" ? ".txt" : ""}`;
        await r.app.invoke("export_workspace_exchange", { projectId: r.project.id, path, format, options: { includeNotes: true, includeBeatComments: true, treatmentLevel: "full", treatmentFormat: "txt" } });
        if (format !== "treatment") {
          const preview = await r.app.invoke("preview_import", { path, format });
          if (preview.scene_count < 2 || preview.chapter_count < 1) throw new Error(`${format} round-trip preview lost project structure`);
        }
      }
      await r.app.invoke("export_workspace_exchange", { projectId: r.project.id, path: `${r.output}/${r.variant}-workspace-treatment.docx`, format: "treatment", options: { includeNotes: true, includeBeatComments: true, treatmentLevel: "five_page", treatmentFormat: "docx" } });
      await r.click('[aria-label="Close export workspace"]');
      await r.js(`localStorage.removeItem(${JSON.stringify(`kindling:export-prototype:v1:${r.project.id}`)}); localStorage.removeItem(${JSON.stringify(`kindling:custom-export:${r.project.id}`)}); delete window.__workspaceExportModule;`);
    },
  },
  {
    id: "20",
    title: "Shortcut filtering, recording and reserved-key feedback",
    async run(r) {
      await r.command("settings");
      await r.wait(el(settings));
      await r.text("Keyboard Shortcuts", `${el(settings)}.querySelector('nav')`);
      await r.wait(`${el(settings)}.querySelector('input[type="search"]')`);
      await r.fill(`${settings} input[type="search"]`, "Find");
      await r.shot(
        "20-01-shortcut-filter",
        "Filtered command bindings, Clear controls and reset action align and wrap cleanly."
      );
      const binding = `${settings} button[aria-label^="Shortcut for "]`;
      await r.click(binding);
      await r.shot(
        "20-02-shortcut-recording",
        "Recording state clearly requests a key combination and explains Escape/Tab."
      );
      await r.js(`const mac=/Mac|iPhone|iPad/.test(navigator.platform);
      ${el(binding)}.dispatchEvent(new KeyboardEvent('keydown',{key:'c',code:'KeyC',metaKey:mac,ctrlKey:!mac,bubbles:true,cancelable:true}));`);
      await r.wait(
        `${el(settings)}.querySelector('[role="alert"]')?.textContent.includes('reserved')`
      );
      await r.shot(
        "20-03-shortcut-reserved",
        "Rejected reserved binding has a readable explanation and leaves the recorder usable."
      );
      await r.js(
        `${el(binding)}.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true,cancelable:true}));`
      );
      await r.click('[data-testid="settings-close"]');
    },
  },
];
