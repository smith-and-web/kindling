# ADR-004: Implement Import Parsers in Native Rust

## Status

Accepted

## Context

Kindling imports outlines from external tools (Plottr, Markdown). These parsers need to:
- Read various file formats (JSON, text)
- Extract hierarchical structure
- Map external data to Kindling's model
- Handle large files efficiently

Options considered:
- **JavaScript parsers in frontend**: Easy to write, runs in webview
- **WASM parsers**: Write once, run in both contexts
- **Native Rust parsers**: Maximum performance, direct file access

File format details:
- **Plottr** (`.pltr`): JSON, can be large with many timeline entries
- **Markdown**: Text with heading-based structure

## Decision

Implement all parsers in native Rust as part of the Tauri backend.

**Reasons:**
1. **Direct file access**: Rust can read files without webview sandboxing
2. **Performance**: Native code for parsing large Plottr files
3. **Consistent model**: Parsers output directly to Rust structs
4. **Error handling**: Rust's `Result` type for robust error handling

## Consequences

### Positive
- Fast parsing even for large files
- Direct integration with database insertion
- Type-safe parsing with Rust structs
- Single source of truth for data model

### Negative
- Adding new import formats requires Rust knowledge
- More complex than a quick JavaScript parser
- Testing requires Rust tooling

### Neutral
- Parser code lives alongside backend code
- Each format has its own module (`parsers/plottr.rs`, etc.)
- Test fixtures needed for each format
