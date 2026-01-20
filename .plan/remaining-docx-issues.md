# Remaining DOCX Issues Implementation Plan

## Overview

Two issues remain open for DOCX export functionality:
- **#99** - Text formatting and special characters
- **#114** - Umbrella issue with remaining items (font selection, spacing options, etc.)

Based on my analysis, the most impactful remaining work is **#99** (preserving text formatting), as the current implementation strips all HTML formatting from prose content, losing italic/bold styling that authors use for emphasis, thoughts, and foreign words.

---

## Issue #99: Text Formatting and Special Characters

### Current Problem

The `strip_html()` function in `export.rs:235` removes ALL HTML tags, discarding formatting:
```rust
fn strip_html(html: &str) -> String {
    // Currently strips <em>, <strong>, etc. completely
}
```

TipTap stores prose as HTML (e.g., `<p>She thought, <em>this can't be real</em>.</p>`), but the export outputs plain text without italics.

### Implementation Tasks

#### Task 1: Replace `strip_html()` with `html_to_docx_runs()`

Create a new function that parses HTML and produces `docx_rs::Run` elements with proper formatting:

**File:** `src-tauri/src/commands/export.rs`

```rust
/// Convert HTML prose content to formatted DOCX runs
///
/// Handles:
/// - <em> / <i> → italic
/// - <strong> / <b> → bold
/// - <p> → paragraph breaks
/// - Plain text → normal runs
fn html_to_docx_runs(html: &str) -> Vec<(String, bool, bool)> {
    // Returns Vec of (text, is_italic, is_bold) tuples
    // These can then be converted to Run objects with proper formatting
}
```

**Approach:** Use a simple state-machine parser or the existing `quick-xml` crate (already a dependency) to walk through HTML tags and track formatting state.

**Estimated complexity:** Medium - requires careful parsing but straightforward logic

#### Task 2: Smart Quote Conversion

Add a text transformation function:

```rust
/// Convert straight quotes to typographic (curly) quotes
fn smartify_quotes(text: &str) -> String {
    // " at word boundary start → "
    // " at word boundary end → "
    // ' as apostrophe → '
    // ' at word start → '
    // ' at word end → '
}
```

**Considerations:**
- Handle edge cases: "Hello," she said → "Hello," she said
- Apostrophes in contractions: don't → don't
- Possessives: John's → John's
- Opening quotes after paragraph start or space

**Estimated complexity:** Medium - regex-based with edge cases

#### Task 3: Em Dash and Ellipsis Normalization

```rust
/// Normalize dashes and ellipses for manuscript format
fn normalize_punctuation(text: &str) -> String {
    // -- or --- → — (em dash)
    // Remove spaces around em dashes: " — " → "—"
    // ... → … (optional, or keep as three periods)
    // Double spaces after periods → single space
}
```

**Estimated complexity:** Low - simple string replacements

#### Task 4: Update `add_beat_to_docx()` to Use New Functions

Modify the beat prose handling to:
1. Parse HTML into formatted runs
2. Apply smart quotes and punctuation normalization
3. Build paragraphs with proper `Run` objects that preserve formatting

**File changes:**
- `src-tauri/src/commands/export.rs` - Update `add_beat_to_docx()` function (~lines 1128-1160)

#### Task 5: Add Unit Tests

- Test HTML parsing with nested tags
- Test smart quote conversion edge cases
- Test em dash normalization
- Test full round-trip: HTML input → formatted DOCX runs

---

## Issue #114: Remaining Items

After #99 is complete, the following items from #114 remain. These are lower priority and could be deferred to a future release:

### Already Complete (can update issue checklist)
- [x] Running headers with author name / title / page number
- [x] Word count on title page
- [x] Author name field (via app settings)
- [x] Contact information for title page
- [x] Chapter heading style options (5 styles implemented)
- [x] Chapter numbering options (word numbers, Arabic numerals)

### Remaining Items (Lower Priority)

#### Font Selection (Times New Roman alternative)
- Add `font_family` field to `DocxExportOptions`
- Add dropdown to ExportDialog.svelte
- Update `create_docx_styles()` to use selected font
- **Complexity:** Low
- **Impact:** Low (Courier New is the SMF standard)

#### Spacing Selection (single, 1.5, double)
- Add `line_spacing` field to `DocxExportOptions`
- Add dropdown to ExportDialog.svelte
- Update all `LineSpacing::new().line(480)` calls to use configurable value
- **Complexity:** Low
- **Impact:** Low (double-spacing is the SMF standard)

#### First Paragraph No-Indent After Scene Breaks
- Track "is first paragraph after break" state in `add_scene_to_docx()`
- Skip first-line indent for that paragraph only
- **Complexity:** Medium
- **Impact:** Medium (proper SMF compliance)

#### Widow/Orphan Control
- Add `.widow_control(true)` to paragraph styles
- The `docx-rs` crate may or may not support this
- **Complexity:** Unknown (depends on crate support)
- **Impact:** Low (Word handles this automatically in most cases)

#### Scene Break Style Options
- Add `scene_break_style` enum: `Hash`, `Asterisks`, `BlankLine`
- Add dropdown to ExportDialog.svelte
- Update scene break rendering in `add_chapter_to_docx()`
- **Complexity:** Low
- **Impact:** Low (# is the SMF standard)

---

## Recommended Implementation Order

### Phase 1: Issue #99 (High Priority)
1. **Task 1:** HTML to DOCX runs parser
2. **Task 2:** Smart quote conversion
3. **Task 3:** Punctuation normalization
4. **Task 4:** Integrate into export
5. **Task 5:** Unit tests

**Rationale:** This is the most impactful change. Authors lose all their intentional formatting (italics for thoughts, emphasis, foreign words) in the current export. Fixing this makes exports actually usable for submission.

### Phase 2: Issue #114 Remaining (Lower Priority)
1. First paragraph no-indent (proper SMF compliance)
2. Scene break style options (nice-to-have)
3. Font selection (nice-to-have)
4. Spacing selection (nice-to-have)
5. Widow/orphan control (if supported)

**Rationale:** These are polish items. The current defaults (Courier New, double-spaced, # scene breaks) are already SMF-compliant.

---

## Files to Modify

### Phase 1
- `src-tauri/src/commands/export.rs`
  - Add `html_to_docx_runs()` function
  - Add `smartify_quotes()` function
  - Add `normalize_punctuation()` function
  - Update `add_beat_to_docx()` to use new functions
  - Add unit tests

### Phase 2 (if approved)
- `src-tauri/src/commands/export.rs`
  - Add new option fields to `DocxExportOptions`
  - Update rendering functions
- `src/lib/types.ts`
  - Add new TypeScript types for options
- `src/lib/components/ExportDialog.svelte`
  - Add UI controls for new options

---

## Questions for Review

1. **Smart quotes:** Should this be an optional toggle, or always applied? Some writers may prefer straight quotes for technical reasons (e.g., certain submission systems).

2. **Phase 2 priority:** Do you want to include any Phase 2 items in this implementation, or defer them entirely?

3. **HTML parsing approach:** Use `quick-xml` (already a dependency) or implement a simple state-machine parser? The former is more robust, the latter is simpler for our limited tag set.
