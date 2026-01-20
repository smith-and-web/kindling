# Importing Projects into Kindling

Kindling supports importing story outlines from multiple sources. This guide explains how to prepare your files for successful import.

## Supported Formats

| Format | Extension | Description |
|--------|-----------|-------------|
| [Plottr](#plottr-pltr) | `.pltr` | Plottr project files |
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

### "Invalid file structure"

- **Plottr**: Ensure the file is valid JSON (not corrupted)
- **Markdown**: Check for encoding issues (should be UTF-8)

### Missing Content After Import

- **No chapters**: Make sure your file has the expected structure markers
- **No scenes**: Scenes require a parent chapter to exist first
- **No beats**: Beats require a parent scene to exist first
- **No characters/locations**: These are only imported from Plottr

### Characters or Locations Not Appearing

- **Plottr**: Characters/places must be defined in the Plottr file
- **Markdown**: Character and location import is not supported (outline only)

---

## Format Comparison

| Feature | Plottr | Markdown |
|---------|--------|----------|
| Chapters | Yes (from beats) | Yes (H1) |
| Scenes | Yes (from cards) | Yes (H2) |
| Beats | Yes (from descriptions) | Yes (lists/paragraphs) |
| Characters | Yes | No |
| Locations | Yes | No |
| Custom Attributes | Yes | No |
| Scene-Character Links | Yes | No |
| Scene-Location Links | Yes | No |
