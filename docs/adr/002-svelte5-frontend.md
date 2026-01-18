# ADR-002: Use Svelte 5 for the Frontend

## Status

Accepted

## Context

The frontend needs a reactive UI framework for building the editor interface. Writers interact heavily with the UI (expanding beats, dragging scenes, editing prose), so reactivity and performance are critical.

Options considered:
- **React**: Most popular, large ecosystem, JSX-based
- **Vue 3**: Composition API, good DX, moderate bundle size
- **Svelte 5**: Compiled, smallest runtime, new runes-based reactivity
- **Solid**: Fine-grained reactivity, JSX-based, smaller community

Key requirements:
- Fast, responsive UI for writing workflows
- Simple state management (not enterprise-scale complexity)
- Good TypeScript support
- Works well with Tauri

## Decision

Use Svelte 5 with the new runes-based reactivity system.

**Reasons:**
1. **Compiled away**: No virtual DOM, minimal runtime overhead
2. **Runes simplicity**: `$state`, `$derived`, `$effect` are intuitive
3. **Single-file components**: `.svelte` files are easy to understand
4. **Bundle size**: Smallest of the major frameworks
5. **Tauri compatibility**: Official Svelte template, good integration

## Consequences

### Positive
- Extremely fast UI updates (no diffing overhead)
- Simple mental model for state management
- Clean component files with scoped styles
- Small bundle contributes to fast app startup

### Negative
- Svelte 5 is newer; some ecosystem packages need updating
- Smaller talent pool than React
- Runes are a new paradigm (learning curve for Svelte 4 developers)
- Less stackoverflow answers for edge cases

### Neutral
- Class-based stores pattern works well but differs from Svelte 4 stores
- Need to be mindful of Svelte 5-specific patterns in documentation
