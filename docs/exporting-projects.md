# Exporting Projects from Kindling

Kindling exports your work to multiple formats so you can move between outlining, drafting, and publishing workflows.

## Export Dialog

Open **Export** from the project toolbar or menu, then choose a format and scope.

![Screenshot: Export dialog](https://raw.githubusercontent.com/smith-and-web/kindling/main/docs/assets/export-dialog.png)

## Supported Formats

| Format            | Output                | Best For                               |
| ----------------- | --------------------- | -------------------------------------- |
| DOCX              | `.docx` file          | Standard Manuscript Format submissions |
| Markdown          | Folder of `.md` files | Plain text workflows or backups        |
| Longform/Obsidian | Index + scene files   | Roundtrip with Obsidian                |
| EPUB              | `.epub` file          | E-readers and ebook previews           |

## Export Scopes

| Scope   | What It Includes          |
| ------- | ------------------------- |
| Project | All chapters and scenes   |
| Chapter | Only the selected chapter |
| Scene   | Only the selected scene   |

## DOCX (Standard Manuscript Format)

DOCX exports are designed for manuscript submissions and include formatting options for:

- Title page (uses app settings and project pen name)
- Chapter heading style and page breaks
- Scene break markers
- Font family and line spacing
- Optional beat markers and scene synopses

## Markdown

Markdown exports create a folder structure like:

```
My Project/
  01 - Chapter One/
    01 - Scene One.md
    02 - Scene Two.md
```

- Beat markers can be included as headings
- You can delete an existing export folder before writing new files

## Longform/Obsidian

Longform exports are optimized for Obsidian + Longform workflows.

**Output layout:**

```
My Project/
  My Project.md          # Longform index
  Scene One.md
  Scene Two.md
  characters/
  locations/
  items/
  objectives/
  organizations/
```

**Scene files include:**

- YAML frontmatter (`type`, `project`, `status`, `characters`, `setting`, `synopsis`)
- Scene title heading and synopsis block
- `<!-- kindling: ... -->` metadata and beats marker

Reference notes are written into folders for characters, locations, items, objectives, and organizations.

## EPUB

EPUB exports include:

- Metadata (title, author, description, language)
- Theme selection (classic, modern, minimal)
- Optional cover image
- Optional beat markers and scene synopses

---

For importing or syncing exported content, see [Importing Projects](importing-projects.md) and [Sync & Reimport](sync-and-reimport.md).
