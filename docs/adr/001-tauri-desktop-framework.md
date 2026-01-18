# ADR-001: Use Tauri as the Desktop Framework

## Status

Accepted

## Context

Kindling needs to be a cross-platform desktop application that works on macOS, Windows, and Linux. Writers expect native-feeling applications with good performance and small install sizes.

The main contenders were:
- **Electron**: Mature, widely used, bundles Chromium (~150MB+ install size)
- **Tauri**: Rust-based, uses system webview (~10MB install size)
- **Native per-platform**: Maximum performance, 3x development effort

Key considerations:
- Kindling is a writing tool that should feel lightweight, not resource-heavy
- The frontend is primarily UI with moderate complexity
- We need file system access for importing outlines
- Security is important since users trust us with their creative work

## Decision

Use Tauri 2.x as the desktop framework.

**Reasons:**
1. **Small bundle size**: ~10MB vs Electron's 150MB+. Writers often have multiple apps open.
2. **Lower memory footprint**: Uses system webview instead of bundled Chromium.
3. **Rust backend**: Enables high-performance parsing and database operations.
4. **Security model**: Explicit permission system, no Node.js attack surface.
5. **Active development**: Tauri 2.x is stable with good documentation.

## Consequences

### Positive
- Small, fast application that feels native
- Rust backend enables safe, concurrent file operations
- Strong type safety across the stack (Rust + TypeScript)
- Good security defaults

### Negative
- Smaller ecosystem than Electron (fewer ready-made solutions)
- Requires Rust knowledge for backend development
- Platform webview differences can cause subtle rendering issues
- Debugging requires understanding both Rust and web tooling

### Neutral
- Team needs to maintain competency in both Rust and TypeScript
- Some Electron-specific npm packages won't work directly
