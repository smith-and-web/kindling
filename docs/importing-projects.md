# Importing Projects into Kindling

Kindling supports importing story outlines from multiple sources. This guide explains how to prepare your files for successful import.

## Supported Formats

| Format | Extension | Description |
|--------|-----------|-------------|
| [Plottr](#plottr-pltr) | `.pltr` | Plottr project files |
| [Scrivener](#scrivener-scriv) | `.scriv` | Scrivener 3 project packages |
| [Markdown](#markdown-md) | `.md` | Plain text markdown outlines |

---

## Plottr (.pltr)

Plottr is a visual story planning tool. Kindling imports the full project structure including characters, locations, and scene cards.

### What Gets Imported

- **Project metadata** - Series name, premise, genre, theme
- **Chapters** - Plottr "beats" become chapters (typically acts)
- **Scenes** - Scene cards with titles and descriptions
- **Characters** - Names, descriptions, and custom attributes (Role, Gender, etc.)
- **Locations** - Places with descriptions and notes
- **Relationships** - Which characters and locations appear in each scene

### File Requirements

- Export from Plottr as a `.pltr` file (this is the native format)
- The file must be valid JSON
- Supports Plottr file format version 2023+

### Notes

- Rich text formatting (bold, italic) is stripped to plain text
- Custom character/location attributes are preserved
- Scene card descriptions become beat content
- Tags are read but not yet displayed in the UI

---

## Scrivener (.scriv)

Scrivener is a popular writing application. Kindling imports the binder structure and synopsis content from Scrivener 3 projects.

### What Gets Imported

- **Project name** - Derived from the `.scriv` folder name
- **Chapters** - Folders in the Draft/Manuscript become chapters
- **Scenes** - Text documents within chapter folders become scenes
- **Beats** - Synopsis text for each scene becomes beat content
- **Characters** - Character sheets from the Research folder
- **Locations** - Location/Setting sheets from the Research folder

### File Requirements

- Must be a Scrivener 3 project (`.scriv` package/folder)
- The project must contain a `.scrivx` index file inside
- Scrivener 2 projects should be upgraded to Scrivener 3 first

### Project Structure

Kindling expects your Scrivener project to follow this structure:

```
MyNovel.scriv/
├── MyNovel.scrivx          <- Project index file (required)
└── Files/
    └── Data/
        ├── [uuid]/
        │   └── synopsis.txt  <- Scene synopsis (optional)
        └── ...
```

### How the Draft Folder Maps to Kindling

```
Draft (or Manuscript)
├── Chapter 1 (Folder)      -> Chapter
│   ├── Scene 1 (Text)      -> Scene + Beat (from synopsis)
│   └── Scene 2 (Text)      -> Scene + Beat
├── Chapter 2 (Folder)      -> Chapter
│   └── Scene 3 (Text)      -> Scene + Beat
└── Standalone Scene (Text) -> Creates its own Chapter + Scene
```

### Tips for Best Results

1. **Use Folders for Chapters** - Each folder directly under Draft becomes a chapter
2. **Use Text Documents for Scenes** - Each text document becomes a scene
3. **Write Synopsis** - The synopsis field (in the Inspector) becomes the beat content
4. **Use Character/Location Sheets** - These are found in the Research folder and will be imported

### Notes

- Only content in the Draft/Manuscript folder is imported as story structure
- Research folder items are scanned for Character Sheets and Location Sheets
- Actual manuscript text is not imported (only synopsis/outline content)
- Nested folders beyond one level are flattened

---

## Markdown (.md)

For users who prefer plain text, Kindling supports a simple markdown outline format.

### What Gets Imported

- **Project name** - Derived from the filename
- **Chapters** - H1 headers (`# Chapter Title`)
- **Scenes** - H2 headers (`## Scene Title`)
- **Beats** - List items or paragraphs under scenes

### File Format

```markdown
# Act One

## The Opening

- Introduce the protagonist
- Establish the ordinary world
- Show the character's flaw

## The Inciting Incident

- Something disrupts the status quo
- The protagonist must respond

# Act Two

## Rising Action

The stakes begin to escalate.

Multiple paragraphs also work as beats.

- Or you can mix list items
- With paragraph content

## The Midpoint

* Asterisk lists work too
* A major revelation occurs
```

### Syntax Reference

| Element | Syntax | Example |
|---------|--------|---------|
| Chapter | `# ` + title | `# Chapter One` |
| Scene | `## ` + title | `## The Beginning` |
| Beat (list) | `- ` or `* ` + content | `- Hero wakes up` |
| Beat (paragraph) | Plain text under a scene | `The sun rose slowly.` |

### Rules and Behavior

1. **Chapters require H1** - Lines starting with `# ` (hash + space) become chapters
2. **Scenes require H2** - Lines starting with `## ` (two hashes + space) become scenes
3. **Scenes need chapters** - H2 headers are ignored if no H1 chapter exists yet
4. **Beats need scenes** - List items and paragraphs are ignored if no scene exists yet
5. **H3+ are treated as beats** - `###` and beyond become paragraph-style beats, not structure
6. **Empty list items are skipped** - A line with just `- ` or `* ` is ignored

### Tips for Best Results

1. **Start with a chapter** - Begin your file with `# Chapter Name`
2. **Keep it simple** - The parser intentionally ignores complex markdown (tables, code blocks, etc.)
3. **One thought per beat** - Each list item or paragraph becomes a separate beat
4. **Use consistent formatting** - Pick either `-` or `*` for lists (both work, but consistency helps readability)

### Special Characters

The parser handles special characters correctly:

- **Unicode** - International characters, accents, emoji all work
- **Quotes and ampersands** - `"quotes"` and `&` are preserved
- **Markdown formatting** - `**bold**` and `*italic*` are preserved as-is (not rendered)

### Edge Cases

| Input | Result |
|-------|--------|
| Empty file | Creates a default "Chapter 1" |
| Only H2 headers (no H1) | Creates default chapter, but scenes are ignored |
| Only list items (no headers) | Creates default chapter, but beats are ignored |
| Only H1 headers | Chapters with no scenes or beats |

---

## Troubleshooting

### "Could not read file"

- Check that the file exists and you have read permissions
- For Scrivener, ensure you're selecting the `.scriv` folder, not a file inside it

### "Invalid file structure"

- **Plottr**: Ensure the file is valid JSON (not corrupted)
- **Scrivener**: Ensure the `.scrivx` file exists inside the `.scriv` package
- **Markdown**: Check for encoding issues (should be UTF-8)

### Missing Content After Import

- **No chapters**: Make sure your file has the expected structure markers
- **No scenes**: Scenes require a parent chapter to exist first
- **No beats**: Beats require a parent scene to exist first
- **No characters/locations**: These are only imported from Plottr and Scrivener

### Characters or Locations Not Appearing

- **Plottr**: Characters/places must be defined in the Plottr file
- **Scrivener**: Characters must use the "Character Sheet" template; locations must use "Setting Sheet"
- **Markdown**: Character and location import is not supported (outline only)

---

## Format Comparison

| Feature | Plottr | Scrivener | Markdown |
|---------|--------|-----------|----------|
| Chapters | Yes (from beats) | Yes (from folders) | Yes (H1) |
| Scenes | Yes (from cards) | Yes (from text docs) | Yes (H2) |
| Beats | Yes (from descriptions) | Yes (from synopsis) | Yes (lists/paragraphs) |
| Characters | Yes | Yes (Character Sheets) | No |
| Locations | Yes | Yes (Setting Sheets) | No |
| Custom Attributes | Yes | No | No |
| Scene-Character Links | Yes | No | No |
| Scene-Location Links | Yes | No | No |
