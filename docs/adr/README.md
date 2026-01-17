# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the Kindling project.

## What is an ADR?

An ADR is a document that captures an important architectural decision made along with its context and consequences. ADRs help new contributors understand *why* the codebase is structured the way it is.

## ADR Index

| ID | Title | Status |
|----|-------|--------|
| [001](001-tauri-desktop-framework.md) | Use Tauri as the desktop framework | Accepted |
| [002](002-svelte5-frontend.md) | Use Svelte 5 for the frontend | Accepted |
| [003](003-sqlite-local-storage.md) | Use SQLite for local data storage | Accepted |
| [004](004-native-rust-parsers.md) | Implement import parsers in native Rust | Accepted |

## Creating a New ADR

1. Copy the template below
2. Create a new file: `NNN-short-title.md`
3. Fill in the sections
4. Add to the index above
5. Submit a PR

### Template

```markdown
# ADR-NNN: Title

## Status

Proposed | Accepted | Deprecated | Superseded by [ADR-XXX](XXX-title.md)

## Context

What is the issue that we're seeing that is motivating this decision or change?

## Decision

What is the change that we're proposing and/or doing?

## Consequences

What becomes easier or more difficult to do because of this change?
```

## Further Reading

- [Documenting Architecture Decisions](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions) - Michael Nygard
- [ADR Tools](https://github.com/npryce/adr-tools)
