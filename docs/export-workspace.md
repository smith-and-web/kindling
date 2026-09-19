# Customize exports with profiles

Use the export workspace to prepare a manuscript for an agent, a reading group,
a website, or another writing app. An **export profile** remembers the format,
content selection, and settings for a particular recipient or routine. Changing
export settings leaves your writing and its formatting in Kindling unchanged.

For a quick export with the standard options, see [Exporting Projects](exporting-projects.md).

## Create your first profile

1. Open your project and choose **File → Export**.
2. Select the **Custom** tile, choose an **Export profile**, and select **Open workspace**.
3. Start with **Agent submission**, **Writing group**, or **Website chapters**.
4. Use **Duplicate profile**, beside the profile dropdown, to make your own copy.
   In **Overview**, give it a useful name, such as “Agency submission” or “Friday readers.”
5. Choose an **Output format** and adjust the settings in the sidebar. The preview
   updates as you work.
6. Select **Save profile** to keep your settings.
7. Select **Export**, choose a destination and a new filename, then use
   **Open export** to open the result.

The starting profiles are editable. Check your recipient's requirements before
using one for a submission.

After you choose Custom, Kindling selects it by default whenever you reopen
Export for that project. It also remembers the last profile you chose in either
the export dialog or the workspace. You can still select a standard format for
an individual export; your Custom preference remains saved.

## Find the settings you need

Use **Find a setting…** to search for terms such as “double spaced,” “scene
separator,” or “running head.” Selecting a result takes you to the setting.
Some settings are available only for particular output formats.

| Section               | What you can change                                                                                                               |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| **Overview**          | Profile name, starting profiles, and a summary of the current selection and format                                                |
| **Content**           | Entire manuscript or selected chapters; scene titles, synopses, and beat headings                                                 |
| **Headings & breaks** | Chapter titles and numbering, Part titles, scene separators, and Word chapter page breaks                                         |
| **Text & page**       | Font, size, line spacing, paragraph spacing, indentation, and alignment; Word paper size and margins                              |
| **Book details**      | Title, author or pen name, subtitle, title page or title block, word count, and contents; Word running headers and ebook metadata |
| **Files & format**    | Filename pattern and options specific to HTML, treatments, or project exports                                                     |

The sidebar shows the sections relevant to the selected format. On smaller
windows, use **Settings** and **Preview** to switch between the two panes.

## Choose what goes into the manuscript

In **Content**, choose **Entire manuscript** or **Selected chapters**. Selected
chapters follow their order in your project. An entire-manuscript profile also
includes eligible chapters you add later.

To start with one chapter or scene, choose **Export** from its menu in the
sidebar, then open Custom. That selection applies to this export. It becomes
part of the profile only if you explicitly save or duplicate the profile.
For a scene export, **Include the rest of its chapter** expands the selection.

The workspace's Word, EPUB, HTML, Markdown, and plain-text exports include saved
prose from the scenes' active writing views. Archived chapters and scenes,
unused scenes, notes, to-dos, discovery notes, and editorial comments are
excluded. You can add scene titles, scene synopses, and beat headings separately.
For scenes written in Page View, included beat headings follow the scene prose.

The workspace loads your saved writing when it opens. If you have made further
changes, let them finish saving and use **Refresh saved manuscript** before
exporting. Unsaved writing is not included in the loaded manuscript.

Longform, Scrivener, novelWriter, and treatment exports use the **whole project**
and their own content rules. They do not use the workspace's chapter selection
or manuscript typography settings.

## Check the preview

For Word, EPUB, and HTML, use the preview's chapter dropdown to inspect a chapter
or choose **Whole selection** to see the title page and contents. This dropdown
changes only what you view: Export still includes the full selection from
**Content**.

Word previews approximate layout. Open the exported document in your word
processor to check pagination and repeating headers. Ebook readers may apply
their own font and spacing preferences.

HTML offers **Read** and **HTML** tabs so you can inspect both the rendered
manuscript and its source. Markdown and plain text show the generated text for
the whole selection. Project formats and treatments show a description of the
output instead of a manuscript preview.

## Prepare a submission manuscript

Start with **Agent submission** and choose **Word manuscript** as the output
format. In **Text & page**, set the requested font, size, line spacing, paper
size, and margins. Set paragraph indentation and, if needed, select **No indent
after a heading or scene break**.

Use **Headings & breaks** for chapter numbering, scene separators, and page
breaks. In **Book details**, set your title and pen name, choose whether to
include a title page and word count, and configure the running header. Word
omits the running header from the title page.

Save a separate profile for each recipient whose requirements differ.

## Prepare an ebook or reading copy

Start with **Writing group** and choose **Ebook / reader copy**. Select the
chapters you want to share and choose whether to include scene titles or a
table of contents.

In **Book details**, enter the title, author, description, and language. Use
**Choose cover** to select a PNG or JPEG image. The cover is included in the
EPUB but is not shown in the manuscript preview.

## Export HTML for a website

Start with **Website chapters** and choose **Web / HTML**. In **Files & format**:

- Choose **Complete HTML document** for a standalone page, or **Body fragment for
  pasting** for content you will insert into another page.
- Choose **Heading 1** or **Heading 2** for the chapter heading element.
- Keep **Include built-in styling** enabled to use your typography settings, or
  turn it off to let your website supply the styling.

Inspect the **HTML** tab before exporting. Both options produce one `.html`
file containing your selected chapters. Embedded images and unsupported
formatting are omitted from manuscript output.

## Export text or move to another writing app

| Output format           | What you receive                                                           |
| ----------------------- | -------------------------------------------------------------------------- |
| **Markdown**            | One `.md` manuscript file with headings and supported text formatting      |
| **Plain text**          | One `.txt` manuscript file without rich-text formatting                    |
| **Longform / Obsidian** | A Longform index, individual scene files, and reference notes              |
| **Scrivener**           | A new `.scriv` project                                                     |
| **novelWriter**         | A project folder, with optional reference notes and beat comments          |
| **Treatment**           | A `.docx` or `.txt` treatment built from your outline, synopses, and beats |

For separate Markdown scene files, use Markdown in the standard export dialog
or choose Longform / Obsidian. The workspace's Markdown option produces a single
manuscript file.

To update an existing Scrivener project with scene matching and backups, select
**Back to export** and choose **Scrivener** in the standard dialog. novelWriter
export is available for prose projects; screenplay projects are not supported.

Treatments offer **One page · overview**, **Five pages · key scenes**, and
**Full · scenes and beats**. These choices set the level of detail; the actual
page count depends on your material.

## Name and save the exported file

In **Files & format**, enter a **Filename pattern**. Use the buttons beneath the
field to insert `{title}`, `{profile}`, or `{date}`. The example below the field
shows the resulting filename. Kindling adds the file extension for you.

For example, `{title}-{profile}-{date}` could produce
`The Letter-Friday readers-2026-09-11.epub`.

Choose a destination each time you export. Workspace exports preserve existing
files and folders, so use a new name for each copy. For Longform and novelWriter,
choose the parent folder; Kindling creates the named project folder inside it.

## Save, reuse, and recover profiles

**Save profile** keeps the current settings. **Export** uses the settings you
see, including changes you have not saved to the profile. Switching profiles
also saves valid changes to the current profile. To try a different arrangement
without changing an existing profile, duplicate it first.

**Revert** returns to the saved settings. Kindling also retains unsaved profile
changes on this device so you can resume after closing the workspace. If an
incomplete draft cannot be recovered, it loads the saved profile instead.

Profiles belong to a project on this device. They are not included in project
transfers or snapshots. Restoring a snapshot does not restore profile settings.
Keep a note of any settings you need to recreate on another device.

## If export is unavailable

- **Nothing selected:** open **Content** and select a chapter containing manuscript
  scenes, or choose **Entire manuscript**.
- **Part of a saved selection is missing:** review the chapter selection after
  deleting or archiving content.
- **A setting is incomplete:** enter a profile name and finish any highlighted
  numeric fields. You can keep editing while the preview remains visible.
- **The destination already exists:** choose a new filename or folder name.
- **A cover image cannot be read:** use **Choose cover** to select the image again.
- **Profiles could not be loaded or saved:** existing saved settings are preserved.
  Avoid clearing app data to resolve the error; keep a note of your settings and
  include the displayed message when asking for help.

For importing or syncing an exported project, see [Importing Projects](importing-projects.md)
and [Sync & Reimport](sync-and-reimport.md).
