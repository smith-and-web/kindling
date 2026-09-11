# Exporting Projects from Kindling

Export your writing for a submission, a reading copy, a website, or another
writing app. Choose a standard format for a quick export, or use **Custom** to
save your own export profiles and adjust settings beside a live preview.

Exporting leaves your writing in Kindling unchanged. Prose exports use the
writing from each scene's selected view—Page View or Beat View—so the same
prose is not included twice. Scenes without beats use their Page View prose.

## Make an export

1. Open your project and choose **File → Export**.
2. Select a format, such as **Word**, **ePub**, or **Markdown**.
3. Adjust the options shown for that format and choose a destination.
4. Select **Export**.

To export a particular chapter or scene, use **Export** in its sidebar menu.
Check the scope shown in the dialog; some project formats always include the
whole project.

## Save custom settings

Select the **Custom** tile, choose a profile, and select **Open workspace**.
The workspace includes starting profiles for agent submissions, writing groups,
and website chapters. You can duplicate a profile, change its settings, and
save it for your next export.

Once you choose Custom, it becomes the remembered choice for that project, along
with your last selected profile. You can still select another format for an
individual export.

See [Customize exports with profiles](export-workspace.md) for the full guide to
content selection, typography, book details, HTML, and saved settings.

## Choose an output format

| Format                  | Output                                                                     | Useful for                                               |
| ----------------------- | -------------------------------------------------------------------------- | -------------------------------------------------------- |
| **Word**                | `.docx` file                                                               | Manuscript submissions and word processors               |
| **ePub**                | `.epub` file                                                               | E-readers and reading copies                             |
| **Markdown**            | Separate scene files in the standard dialog; one manuscript file in Custom | Text-based writing tools                                 |
| **HTML**                | One `.html` document or fragment, through Custom                           | Websites and publishing tools                            |
| **Plain text**          | One `.txt` manuscript, through Custom                                      | Unformatted text                                         |
| **Longform / Obsidian** | Index, scene files, and reference notes                                    | Working in Obsidian with Longform                        |
| **Scrivener**           | `.scriv` project                                                           | Moving work to Scrivener or updating an existing project |
| **novelWriter**         | Project folder                                                             | Moving prose projects to novelWriter                     |
| **Treatment**           | `.docx` or `.txt` file                                                     | Sharing an outline or story summary                      |

## Word manuscripts

The standard Word export offers manuscript formatting options for a title page,
chapter headings, page breaks, scene separators, font, and line spacing. You can
also include beat headings and scene synopses.

Use Custom for additional control over font size, paragraph spacing and
indentation, margins, paper size, running headers, and contents. Save a profile
for each recipient's submission requirements.

## EPUB reading copies

In the standard dialog, set the book title, author, description, and language,
choose a theme, and optionally add a cover image. Beat headings and scene
synopses can also be included.

Use Custom to save these book details with your content selection and typography
preferences. Ebook readers may override font and spacing settings, so check the
export in your intended reading app.

## Markdown and plain text

Standard Markdown export creates a folder for the selected chapters and scenes:

```text
My Project/
  01 - Chapter One/
    01 - Scene One.md
    02 - Scene Two.md
```

You can include beat headings. If you select the option to delete an existing
export folder, its contents will be removed before the new files are written.
Use a separate destination to keep an earlier copy.

Custom Markdown and plain-text exports each produce a single manuscript file.
Markdown retains supported formatting such as emphasis and headings; plain text
removes rich-text formatting. Both show the generated text in the workspace.

## Longform / Obsidian

Longform exports create an index file, individual scene files, and reference
notes grouped into folders for characters, locations, items, objectives, and
organizations. Scene files include titles, synopses, and details used to preserve
your project's structure when you bring changes back into Kindling.

Keep the generated index and scene files together when moving the export.
See [Sync & Reimport](sync-and-reimport.md) for working with a linked source.

## Scrivener and novelWriter

Choose Scrivener to create a new project. To update an existing Scrivener project,
use its options in the standard export dialog, review the scene matches, and
choose whether to make a backup. The Custom workspace creates new Scrivener
projects only.

novelWriter export creates a project folder with options for reference notes and
beat comments. Keep beat comments enabled if you want to preserve beat boundaries
for later sync. novelWriter export supports prose projects, not screenplays.

Within Custom, Longform, Scrivener, and novelWriter always export the whole
project. Manuscript chapter selections and typography settings do not apply to
these project formats.

## Treatments

A treatment summarizes the whole project's outline, synopses, and beats. Choose
an overview, key-scene summary, or full scene-and-beat treatment, then export it
as Word or plain text. The one-page and five-page choices describe the level of
detail; actual page counts depend on your content.

For importing or syncing exported content, see [Importing Projects](importing-projects.md)
and [Sync & Reimport](sync-and-reimport.md).
