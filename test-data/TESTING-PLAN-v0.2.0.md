# Kindling v0.2.0 Testing Plan

## Quick Start

```bash
cd /Users/josh.smith/Claude/kindling
cargo tauri dev
```

## Test Files

| File | Purpose |
|------|---------|
| `test-data/simple-story.pltr` | Initial import (3 chapters, 6 scenes) |
| `test-data/simple-story-updated.pltr` | Reimport testing (adds chapter, scene, beats + updates titles) |

---

## Feature #38: Beat-Level Prose Editing

### Setup
1. Import `simple-story.pltr` as a new project
2. Select "Act 1" chapter → "The Beginning" scene

### Tests

| # | Test | Steps | Expected |
|---|------|-------|----------|
| 1 | Expand beat | Click any beat header | Beat expands, shows textarea |
| 2 | Collapse beat | Press `Escape` or click header again | Beat collapses |
| 3 | Write prose | Type in textarea, wait 1 second | "Saving..." then "Saved" indicator appears |
| 4 | Prose persists | Navigate away, come back to scene | Your prose is still there |
| 5 | Multiple beats | Expand beat 1, then click beat 2 | Beat 1 collapses, beat 2 expands |

### Pass Criteria
- [ ] Beats expand/collapse on click
- [ ] Textarea shows existing prose
- [ ] Auto-save works (500ms debounce)
- [ ] Save indicator shows status
- [ ] Escape key closes beat
- [ ] Prose persists across navigation

---

## Feature #15: Create New Chapters and Scenes

### Setup
1. Open any imported project

### Tests

| # | Test | Steps | Expected |
|---|------|-------|----------|
| 1 | New chapter button | Click "+ New Chapter" at bottom of sidebar | Inline input appears |
| 2 | Create chapter | Type "Test Chapter", press Enter | Chapter created, expands, shows in list |
| 3 | Cancel chapter | Click "+", type something, press Escape | Input disappears, no chapter created |
| 4 | New scene button | Expand a chapter, click "+ New Scene" | Inline input appears |
| 5 | Create scene | Type "Test Scene", press Enter | Scene created, selected, shown in main panel |
| 6 | Cancel scene | Click "+", type something, press Escape | Input disappears, no scene created |
| 7 | Blur cancel | Start creating, click elsewhere | Input disappears |

### Pass Criteria
- [ ] New chapter button visible at bottom of chapters list
- [ ] New scene button visible when chapter is expanded
- [ ] Enter submits, Escape cancels
- [ ] New items appear immediately in list
- [ ] New chapter auto-expands
- [ ] New scene auto-selects

---

## Feature #14: Drag-and-Drop Reordering

### Setup
1. Open project with multiple chapters and scenes

### Tests

| # | Test | Steps | Expected |
|---|------|-------|----------|
| 1 | Drag handle visible | Hover over chapter row | Grip icon (⋮⋮) appears on left |
| 2 | Drag chapter | Drag "Act 2" above "Act 1" | Drop indicator line shows, order changes on drop |
| 3 | Chapter order persists | Close and reopen project | New order maintained |
| 4 | Drag scene | Expand chapter, drag scene to new position | Order changes |
| 5 | Scene order persists | Navigate away and back | New order maintained |
| 6 | Visual feedback | While dragging | Dragged item is semi-transparent |
| 7 | Drop indicator | Drag over another item | Blue line shows where item will drop |

### Pass Criteria
- [ ] Drag handles appear on hover
- [ ] Chapters can be reordered
- [ ] Scenes can be reordered within a chapter
- [ ] Visual feedback during drag (opacity, drop line)
- [ ] Order persists after reload

---

## Feature #16: Delete Chapters and Scenes

### Setup
1. Open project with content you're okay deleting (create test chapters/scenes first)

### Tests

| # | Test | Steps | Expected |
|---|------|-------|----------|
| 1 | Delete button visible | Hover over chapter | Trash icon appears |
| 2 | Delete chapter dialog | Click trash on chapter with scenes | Dialog shows "X scenes, Y beats will be deleted" |
| 3 | Confirm delete | Click "Delete" in dialog | Chapter and all contents removed |
| 4 | Cancel delete | Click "Cancel" in dialog | Nothing deleted |
| 5 | Delete scene | Hover scene, click trash | Dialog with beat count |
| 6 | Empty chapter delete | Delete chapter with no scenes | Simpler confirmation message |
| 7 | Delete persists | Close and reopen project | Deleted items stay deleted |

### Pass Criteria
- [ ] Trash icon appears on hover
- [ ] Confirmation dialog shows counts
- [ ] Delete actually removes items
- [ ] Cancel doesn't delete anything
- [ ] Selection clears if deleted item was selected

---

## Feature #40: Re-import to Update Project

### Setup
1. Import `simple-story.pltr`
2. Write some prose in at least 2 beats (this will test prose preservation)
3. **Copy** `simple-story-updated.pltr` **over** `simple-story.pltr`:
   ```bash
   cp test-data/simple-story-updated.pltr test-data/simple-story.pltr
   ```

### What's Different in Updated File
- "Act 2" → "Act 2 - UPDATED TITLE"
- "The Beginning" → "The Beginning - UPDATED"
- New chapter: "Act 4 - NEW CHAPTER"
- New scene: "NEW SCENE - Preparation"
- New beat in "The Beginning"
- New scene: "Epilogue" in Act 4

### Tests

| # | Test | Steps | Expected |
|---|------|-------|----------|
| 1 | Reimport button visible | Open project with source_path | RefreshCw icon next to "All Projects" |
| 2 | No button without source | Create project from scratch | No reimport button |
| 3 | Click reimport | Click refresh icon | Spinner shows while processing |
| 4 | Summary dialog | Wait for completion | Dialog shows added/updated counts |
| 5 | Titles updated | Check "Act 2" chapter | Now shows "Act 2 - UPDATED TITLE" |
| 6 | New chapter added | Look for "Act 4" | New chapter visible in list |
| 7 | New scene added | Expand Act 1 | "NEW SCENE - Preparation" visible |
| 8 | **Prose preserved** | Check beats where you wrote prose | **Your prose is still there!** |
| 9 | No changes | Click reimport again | "No changes detected" message |

### Pass Criteria
- [ ] Reimport button only shows for imported projects
- [ ] Spinner animation during reimport
- [ ] Summary dialog shows correct counts
- [ ] New chapters/scenes appear
- [ ] Titles update correctly
- [ ] **CRITICAL: User prose is NOT overwritten**

---

## Quick Smoke Test (5 minutes)

For fast verification that nothing is broken:

1. **Start app**: `cargo tauri dev`
2. **Import**: Open `test-data/simple-story.pltr`
3. **Create**: Add one chapter, add one scene to it
4. **Reorder**: Drag the new chapter to top
5. **Beat edit**: Click a beat, type "Test prose", wait for "Saved"
6. **Delete**: Delete the chapter you created
7. **Reimport**: Copy updated file over original, click refresh button
8. **Verify**: Check that summary shows updates, prose preserved

---

## Troubleshooting

### App won't start
```bash
cd /Users/josh.smith/Claude/kindling
cargo clean
cargo tauri dev
```

### Database issues (stale data)
```bash
# Find and remove the database
rm ~/Library/Application\ Support/com.kindling.dev/kindling.db
```

### Reset test files
```bash
cd /Users/josh.smith/Claude/kindling/test-data
git checkout simple-story.pltr
```
