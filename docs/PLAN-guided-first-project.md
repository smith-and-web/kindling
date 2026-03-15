# Guided First Project — Implementation Plan

**Branch:** `feature/guided-first-project`  
**Issue:** [#55 US-1.1-4: Guided First Project](https://github.com/smith-and-web/kindling/issues/55)  
**Status:** Planning

---

## 1. Acceptance Criteria (from #55)

- [ ] Optional "Guided Setup" for new users (detect first launch)
- [ ] Step-by-step project creation: choose source → select file → preview → import
- [ ] Contextual tooltips explaining each area on first visit
- [ ] Sample project available to explore before creating own
- [ ] "Quick Start" documentation accessible from help menu
- [ ] Can skip guidance at any point for experienced users
- [ ] Preference to disable guidance persists

---

## 2. Current State

**Already implemented:**

- First launch detection via `kindling:onboardingCompleted` in localStorage
- Onboarding flow: welcome → tour-sidebar → tour-editor → tour-references → import
- Skip at any point; completion persists
- Import options (Plottr, Markdown, Longform, yWriter) on final step

**Gaps:**

- No sample project — users must import their own file
- Tour is conceptual (mockups); import step is abrupt
- No guided "choose source → select file → preview → import" wizard
- No Quick Start docs in help menu
- No explicit "disable guidance" preference (only completion)

---

## 3. Proposed Implementation

### Phase A: Sample Project (foundation) ✅ DONE

**Goal:** Bundle a minimal sample project so users can explore Kindling without importing.

| Task | Description                                                             | Complexity |
| ---- | ----------------------------------------------------------------------- | ---------- |
| A1   | Create sample project data (Markdown or embedded JSON)                  | Low        |
| A2   | Add `create_sample_project` Tauri command                               | Medium     |
| A3   | Add "Try sample project" option to onboarding welcome/import step       | Low        |
| A4   | Wire sample project into StartScreen (for users who skipped onboarding) | Low        |

**Sample project content (minimal):**

- 1 chapter, 2–3 scenes, 2–3 beats per scene
- 1–2 characters, 1 location
- Short synopses and beat content
- Source: embedded in app (no external file)

**Technical approach:**

- Store sample as static JSON in Rust (or bundled resource)
- `create_sample_project()` → inserts into DB, returns `Project`
- Project uses `SourceType::Markdown` with `source_path: null` (or a special marker)

---

### Phase B: Guided Import Wizard ✅ DONE

**Goal:** Step-by-step flow: choose source → select file → preview → import.

| Task | Description                                                               | Complexity |
| ---- | ------------------------------------------------------------------------- | ---------- |
| B1   | Add "guided-import" onboarding step (or replace "import" with multi-step) | Medium     |
| B2   | Step 1: Choose source (Plottr / Markdown / Longform / yWriter)            | Low        |
| B3   | Step 2: File picker (filtered by chosen format)                           | Low        |
| B4   | Step 3: Preview (show chapter/scene count, maybe first few items)         | Medium     |
| B5   | Step 4: Confirm import → run import → complete onboarding                 | Low        |

**Preview step:** Call existing `get_sync_preview` or add lightweight `preview_import(path, format)` that returns structure without full import.

---

### Phase C: Contextual Tooltips & Preferences

**Goal:** First-visit tooltips and persistent guidance preference.

| Task | Description                                                                 | Complexity |
| ---- | --------------------------------------------------------------------------- | ---------- |
| C1   | Add `kindling:guidanceEnabled` preference (default true for first launch)   | Low        |
| C2   | Expose in Kindling Settings: "Show guidance tips" toggle                    | Low        |
| C3   | Contextual tooltips on first visit to sidebar, scene panel, references      | Medium     |
| C4   | Tooltips only show when `guidanceEnabled` and not yet seen (per-area flags) | Low        |

**Storage keys:**

- `kindling:guidanceEnabled` — boolean
- `kindling:tooltipSeen:sidebar` (etc.) — optional, or derive from onboarding completion

---

### Phase D: Quick Start Documentation ✅ DONE

**Goal:** In-app Quick Start accessible from help menu.

| Task | Description                                                          | Complexity |
| ---- | -------------------------------------------------------------------- | ---------- |
| D1   | Create Quick Start content (Markdown or Svelte)                      | Low        |
| D2   | Add Help menu item "Quick Start" → opens dialog or panel             | Low        |
| D3   | Content: import formats, sidebar, beats, references, discovery notes | Low        |

---

## 4. Recommended Order

1. **Phase A** — Sample project (unblocks "explore before importing")
2. **Phase B** — Guided import wizard (core UX improvement)
3. **Phase D** — Quick Start docs (low effort, high value)
4. **Phase C** — Tooltips & preferences (polish)

---

## 5. Dependencies & Risks

| Item           | Depends on                            | Risk                                         |
| -------------- | ------------------------------------- | -------------------------------------------- |
| Sample project | None                                  | Low — similar to existing import flow        |
| Guided import  | Existing import commands              | Low                                          |
| Preview step   | May need new `preview_import` command | Medium — check if sync preview can be reused |
| Tooltips       | UI store, guidance preference         | Low                                          |

---

## 6. Out of Scope (for this branch)

- Projects from scratch (v1.2)
- Full documentation site
- Video tutorials

---

## 7. Files to Touch

| Area        | Files                                                               |
| ----------- | ------------------------------------------------------------------- |
| Backend     | `src-tauri/src/commands/` (new `sample_project.rs` or in `crud.rs`) |
| Sample data | `src-tauri/src/` or `src-tauri/resources/`                          |
| Onboarding  | `src/lib/components/Onboarding.svelte`                              |
| StartScreen | `src/lib/components/StartScreen.svelte`                             |
| UI store    | `src/lib/stores/ui.svelte.ts`                                       |
| Settings    | `src/lib/components/KindlingSettingsDialog.svelte`                  |
| Help menu   | `src/App.svelte` or menu component                                  |
| Quick Start | New `QuickStartDialog.svelte` or similar                            |
