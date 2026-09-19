# Kindling — the Press Design Guide

The canonical specification for how Kindling looks. Every surface — the marketing
site, the desktop app, docs, social — consumes this. Nothing redefines it.

This document is written to be **followed literally**, including by an AI coding
assistant. Rules are stated as numbers and prohibitions rather than adjectives,
because adjectives get ignored and numbers do not.

---

## 0. The commitment

**Press is paper-first, literary, restrained, and editorial.**

Commit to that before writing markup. The failure mode for this project is not
ugliness — it is *defaulting*: the centred-band layout of hero → content band →
alternating band → closing CTA, in a neutral sans, with a grid of rounded cards.
That layout is what every tool and every model produces when left alone. It is
competent and it is anonymous.

Press means, concretely:

- Warm paper stock with visible grain — not a flat screen fill.
- A serif display face and a *separate* serif reading face, with a sans reserved
  strictly for things you operate.
- Asymmetric, column-based composition. Text sits in a measured column; headings
  and figures break out of it.
- Hairlines and whitespace doing the work that borders, cards and shadows do
  elsewhere.
- Terracotta appearing once or twice per view. If a page reads as orange, it is
  wrong.

**The test:** if a page could be dropped onto another product's website without
looking out of place, it is not Press yet.

---

## 1. Files and governance

| File | What it is |
|---|---|
| `tokens.css` | The design tokens as CSS custom properties. **Canonical.** Everything derives from this. |
| `tokens.json` | A resolved, flat mirror for tooling (scripts, Figma import). |
| `components.css` | Base, component, and composition styles built on the tokens. Holds no hard-coded colours or font sizes. |
| `DESIGN_GUIDE.md` | This file. The rules the CSS cannot express. |
| `style-guide.html` | Rendered reference. Links the live CSS, so it always reflects the current system. The sign-off surface. |

`design-system/` in **`brand-assets`** is the single source of truth. It syncs
one-way into consuming repos (`kindling-splash/src/styles/`) via
`npm run sync:design-system`.

- **Never hand-edit a synced copy.** Change it here and re-sync.
- **Never free-hand a hex, font family, or font size** anywhere. If you are
  reaching for a raw value, the token is missing — add it here.
- Components read **semantic** tokens (`--color-accent`, `--color-text-muted`,
  `--font-body`), never the raw palette (`--terracotta`, `--ink-2`). The raw
  layer exists so light/dark swap in one place; the semantic layer is the API.

Consuming it is two imports, tokens first:

```css
@import "tokens.css";      /* defines the custom properties */
@import "components.css";  /* styles that read them */
```

---

## 2. Colour

Paper-first. Ink for text, terracotta for accents, used sparingly. The palette is
warm and low-contrast by design. Depth comes from **raised** (lighter than paper)
and **sunken** (darker) surfaces plus hairline borders — not from heavy shadows.

| Role | Light | Dark | Token |
|---|---|---|---|
| Paper — page background | `#F4EFE6` | `#1E1A16` | `--color-bg` |
| Raised — cards, panels, inputs | `#FBF8F1` | `#26211B` | `--color-surface` |
| Sunken — alternating bands | `#ECE4D6` | `#181410` | `--color-surface-sunken` |
| Ink — primary text | `#231D18` | `#E8E0D4` | `--color-text` |
| Ink-2 — secondary text | `#6B635B` | `#A89C8C` | `--color-text-muted` |
| Terracotta — fills, borders | `#B5532E` | `#E08A5C` | `--color-accent` |
| Accent text — small text, links | `#9E3D1B` | `#E08A5C` | `--color-accent-text` |
| Ember — flame inner, decorative | `#E0612C` | `#F0A878` | `--color-flame-inner` |
| Hairline | `rgba(35,29,24,.13)` | `rgba(232,224,212,.13)` | `--color-border` |
| Success | `#356B4D` | `#5FA37E` | `--color-success` |
| Error | `#9E3E36` | `#E58B7F` | `--color-error` |
| Warning | `#775600` | `#D8AB5F` | `--color-warning` |
| Info | `#386078` | `#78ABC4` | `--color-info` |
| Accent wash — highlight fills | `rgba(181,83,46,.08)` | `rgba(224,138,92,.12)` | `--color-accent-wash` |
| Nav scrim — translucent navbar | `rgba(244,239,230,.85)` | `rgba(30,26,22,.85)` | `--color-nav-scrim` |

### Hard contrast rules

Terracotta `#B5532E` is for **fills, borders and icons — never body text on
paper.** It lands at 4.32:1, under AA. Small accent text and links use
`--color-accent-text` (`#9E3D1B`), which clears AA. Paper-coloured text *on* a
terracotta fill is fine (~5.8:1), which is why buttons work.

**Success green is subject to the same arithmetic.** It is used as *text* — the
comparison-table checks — so it must clear AA on every ground it lands on,
including the sunken band. At `#3E7C5A` it was 4.33:1 on paper and 3.93:1 on
sunken, i.e. the exact failure the terracotta rule exists to prevent, missed
because nobody wrote it down. It is now `#356B4D` (4.95:1 at worst).

**Never set opacity on text to make it quieter.** Use `--color-text-muted`,
which is contrast-checked. Opacity silently multiplies against the background
and is how the download size text reached 2.98:1 and the comparison crosses
2.20:1.

### Accent discipline

**One or two terracotta moments per view.** A primary button, or one rule, or one
active state — not all three. Count them before shipping. Terracotta on every
card eyebrow is the single easiest way to make Press look like a template.

On editorial surfaces, `--color-success` is **only** for comparison-table
checks. It is not a hero badge or decorative colour. The application carve-out
in section 10 permits status colours for actual system feedback.

---

## 3. Typography

Three families, each with one job:

- **Fraunces** (`--font-display`) — headings only. Optical sizing is on, so it
  firms up large and stays graceful small.
- **Newsreader** (`--font-body`) — reading prose: paragraphs, stand-firsts,
  captions, callouts, the footer tagline. This is the literary voice, and it is
  what makes the system feel like a tool built for novelists.
- **Inter** (`--font-ui`) — everything functional: nav, buttons, form controls,
  table headers, labels, eyebrows, metadata.

**The test when in doubt:** *is this meant to be read, or operated?* Read →
Newsreader. Operated → Inter. Titled → Fraunces.

### Scale

| Token | Size | Use |
|---|---|---|
| `--text-hero` | `clamp(2rem, 6vw, 3rem)` — 32→48 | The one page-opening statement |
| `--text-h1` | `clamp(2rem, 5.5vw, 2.75rem)` — 32→44 | One display moment per page |
| `--text-h2` | `clamp(1.5rem, 4vw, 2rem)` — 24→32 | Section titles |
| `--text-h3` | 20 | Card and row titles |
| `--text-body-lg` | 18 | Stand-firsts, lead paragraphs |
| `--text-body` | 17 | Reading prose |
| `--text-base` | 16 | Form controls, small card labels |
| `--text-ui` | 15 | Nav, buttons, table cells |
| `--text-small` | 14 | Captions, metadata, folios |
| `--text-eyebrow` | 12 | Uppercase labels |

### Hierarchy

Build a real descent **within** each page: one `--text-hero` or `--text-h1`
display moment, then `--text-h2` section titles, then `--text-h3` row titles. A
big hero followed by a flat plateau of identical section titles is SaaS
hierarchy, not editorial hierarchy.

Let Fraunces be **large and quiet** rather than medium and animated.

**Long-form articles use the same scale.** Post title `--text-h1`, section
headings `--text-h2`, sub-headings `--text-h3`, body `--text-body`. Blog and
docs templates do not get their own type scale, their own `clamp()`, or their
own measure — they inherit these. If long-form needs a step the scale lacks,
add it here rather than inventing one locally.

### Prose colour

Primary reading prose is set in `--color-text` (ink). `--color-text-muted` is for
captions, metadata, and genuinely secondary lines — **never for the main read.**
Secondary ink for body copy is what makes a page feel washed out and generic.

`--text-base` (16px) is the floor for form controls. iOS Safari zooms on focus
below 16px, so do not "tidy" inputs down to `--text-ui`.

---

## 4. Composition

This is the half of the system that CSS tokens cannot carry, and the half that
actually distinguishes Press. Skin without skeleton produces a paper-coloured
SaaS page.

### The frame

- `--page-frame` — 1120px outer editorial frame.
- `--page-gutter` — 32px side padding, 24px below 640px.
- Applied by `.editorial`.

### The measure

- `--measure` — **36rem (576px), ≈85 characters** of Newsreader at 17px.
  Measured, not estimated: Newsreader's average character width at 17px is
  6.81px, and its `0` glyph — one CSS `ch` — is 8.50px, so 576px is 68`ch`.
  Do not reason about the measure in `ch`; the two numbers differ by a quarter.
- **All reading prose is capped at the measure.** 85 characters sits above the
  classic 60–75 recommendation and below the ~100 where a line stops being
  comfortable to track. It is a deliberate compromise: tighter than the 672px
  this system originally shipped, wide enough that long-form pages did not need
  reflowing around a much narrower column.
- **Headings, figures, tables and pull-quotes break out wider** than the prose
  column. That contrast between a narrow text column and wider breakouts is the
  editorial signature — without it you just have a narrow page.

### Alignment

- **Reading prose is left-aligned.** Always.
- **Never centre a paragraph longer than one line** of reading prose. A short
  label inside a real card is not prose and may be centred.
- Centring is reserved for genuinely short, ceremonial moments: the closing CTA
  line, the footer tagline, a one-line eyebrow. It is a gesture, not a default.

### Sequence, not grid

Feature content reads as a **sequence**, not a matrix of identical tiles:

- `.feature-seq` — alternating figure-left/prose-right, then reversed.
- Rows separated by a 1px `--color-border` hairline at `--space-3xl` (80px).
- Each row carries a print folio (`01`, `02`, …) set in `--text-small`,
  `--color-text-muted`, at `--tracking-eyebrow`.
- Card grids of equal tiles produce ragged heights and dead space beside
  text-only tiles. Use `.spotlight-list` — hairline-separated rows — instead.

Keep bordered cards **only where a card is semantically real**: the download
platform cards, the detail table. Not for every idea on the page.

### Bands and surfaces

- Bands step **down** (`--color-surface-sunken`); cards step **up**
  (`--color-surface`). Never both at once.
- Alternate paper → sunken → paper so sections read as planes.
- `.content-section.alt` breaks out full-bleed. A sunken band that stops at
  900px reads as an inset box, not a band.

### Vertical rhythm

80px bands, 120–140px page tops (`--space-3xl` / `--space-4xl` / `--space-5xl`).
**This is correct and is not the problem** — the vertical cadence was never what
made the old layout generic. Do not compress it to fit more in.

---

## 5. Named devices

These are the specific, ownable gestures of Press. They are *licensed
exceptions* to the restraint rules below — use them deliberately, and do not
invent new ones without adding them here.

**Paper grain.** A monochrome fractal-noise tile (`--grain-tile`) at
`--grain-strength: 0.14`, fixed behind all content at `z-index: -1`, with
`.has-grain` re-asserting it inside opaque sunken bands. This is *stock*, not
decoration — it is why the page reads as paper. Tune the strength; never
hand-edit the encoded tile.

**Mounted print.** Product figures get a raised-surface mat (10px in sequences,
12px in a hero), a hairline ring, and `--shadow-md` — so a screenshot reads as a
print mounted on board rather than an image flush to the page.

**Print folio.** `counter(folio, decimal-leading-zero)` in the gutter of a
feature sequence. A quiet typographic ornament that gives a sequence rhythm.

**Accent rule.** `.accent-rule` — a 48×3px terracotta bar. Often *the* one
terracotta moment in a section. One per section at most.

**Callout pull-quote.** `.callout` — terracotta left rule, italic Newsreader,
raised surface. The most Press element in the system. Use it to break up long
runs of prose and give the eye rest.

### Where shadow is allowed

Prefer a 1px hairline to a shadow. Shadow is licensed in exactly **three** places:

1. Mounted-print figures.
2. The closing CTA panel, where a card deliberately steps up off the page.
3. A floating overlay that must read as above the page — the mobile nav
   dropdown, a dialog. Elevation is doing real work here, not decoration.

Everywhere else, use `--color-border`.

---

## 6. Motion

Restraint applies to motion as much as to colour.

- Scroll reveals use `animation-timeline: view()` with a fade, a small rise and a
  slight scale. Nothing longer than ~0.7s.
- Everything is wrapped in `@media (prefers-reduced-motion: no-preference)`, with
  an `@supports not (animation-timeline: view())` fallback that shows content
  immediately.
- Hover transitions are 0.1–0.2s.
- **No per-character typing effects, no blinking cursors, no auto-looping
  product video.** If motion is wanted on a headline, fade the whole line once.

---

## 7. Hard rules

Checkable. If you cannot answer these by looking, the page is not finished.

1. Reading prose is capped at `--measure` (36rem).
2. Reading prose is `--color-text`, not `--color-text-muted`.
3. No paragraph of **reading prose** longer than one line is centred. Short
   labels inside a semantically-real card may be centred.
4. Terracotta appears at most **twice** per viewport, counting deliberate
   moments — fills, rules, badges, icons. Inline links set in
   `--color-accent-text` are the system's link colour, not accent moments,
   and do not count.
5. On editorial surfaces, `--color-success` appears only in a comparison table.
6. Every colour and font size resolves to a token — no raw hex, no raw px.
7. Shadow appears only on a mounted-print figure, the closing CTA panel, or a
   floating overlay.
8. Each page has exactly one `--text-hero`/`--text-h1` display moment.
9. Body text on paper is never `--color-accent`.
10. Form controls are at least `--text-base` (16px).
11. Bands are full-bleed; cards are not.
12. Icons are 1px line SVGs, monochrome.

---

## 8. Never

Enumerated prohibitions. These are the specific artifacts that pull a page back
toward the generic default.

- **Never** use emoji as interface icons.
- **Never** centre a multi-line paragraph of reading prose.
- **Never** set primary reading prose in muted ink.
- **Never** put a terracotta eyebrow on every card in a grid.
- **Never** use green outside a comparison-table check.
- **Never** use a gradient, a glow, or a colour transition as decoration.
- **Never** use a typewriter or per-character reveal.
- **Never** autoplay a looping product video.
- **Never** ship a full-window screenshot where a cropped detail would show the
  same thing — especially on mobile, where a shrunk window is illegible.
- **Never** use Inter for reading prose, or Newsreader for controls.
- **Never** use a system sans (Arial, Helvetica, Roboto, Open Sans) anywhere.
- **Never** hand-edit a synced copy of `tokens.css` or `components.css`.
- **Never** stack a card grid directly on another card grid.

---

## 9. Dark mode

Opt-in via `[data-theme="dark"]` on a root element. `tokens.css` defines the full
dark palette; components inherit it automatically because they only read semantic
tokens — which is precisely why rule 6 above matters. A single hard-coded colour
silently breaks dark mode, and it will not show up in light-mode review.

**The web is light-only, by decision.** The marketing site and the docs are one
light Press surface and the Starlight theme picker is removed. Do not add a
dark toggle to either without that decision being revisited.

**The dark palette stays anyway, because the desktop app uses it.** The app
supports light, dark and system, and is moving to light-by-default in the
reskin — dark remains a supported choice there. Do not delete the dark tokens
as "unexercised": their consumer is the app, not this website.

---

## 10. Application surfaces

The desktop app inherits the Press palette, typography jobs, contrast rules,
reading measure, grain, mounted-print treatment, and hairline-first separation.
It also supports light, dark, and system themes, with light as the default. Dark
is a permanent supported choice even though the marketing site is light-only.

The app is denser and stateful, so these editorial rules are explicitly relaxed:

- **Accent count:** an active scene, selected tag, focus treatment, and primary
  action may coexist. Terracotta still communicates action or selection; it is
  never ambient decoration.
- **Shadow:** dialogs, menus, tooltips, the command palette, toasts, and other
  floating layers use `--shadow-overlay`. In-flow separation remains a hairline.
- **Page composition:** the one-display-heading, full-bleed-band, sequence-not-grid,
  and marketing-page rhythm rules do not apply to workspace layouts.
- **Status colour:** success, error, warning, and info are allowed for real
  application state. They never decorate content or replace plain hierarchy.

Application implementation rules:

1. Use the `--z-*` scale, including the paired command/guidance backdrop and
   surface tokens. Never free-hand a stacking integer in a consumer.
2. Dialog and overlay backdrops use `--color-overlay-scrim`; menus and dialogs
   use the shared overlay shadow.
3. Inputs, selects, and textareas use the control tokens for normal, hover,
   focus, invalid, and disabled states. Their text is never below `--text-base`.
4. Disabled controls use the disabled tokens, not element or text opacity.
5. Selection and scrollbar colours come from their application tokens.
6. The editor sheet is `.app-prose-sheet`: warm paper with grain, a hairline
   ring, and mounted-print shadow in both light and dark application chrome.
7. Editor and other reading prose use Newsreader at `--text-body`, ink, and a
   hard maximum width of `--measure` (36rem). Do not substitute a `ch` measure.
8. Tag colours come from the canonical tag palette and are user-authored
   metadata. CSS UI uses the theme-aware `--color-tag-*` semantics; persisted
   user choices use the stable light `--tag-*` values. Tags are not
   interchangeable with status colours.

## 11. Component inventory

`style-guide.html` is the rendered inventory and the sign-off surface. It links
the live `tokens.css` and `components.css`, so it always reflects the current
system. Open it in a browser rather than reading the CSS when you want to know
what exists.

---

## 12. Open decisions

- **Sunken (`#ECE4D6`) and Success (`#356B4D`) were derived, not taken from the
  brand pack.** They have been in use since the Press reskin and are treated as
  canonical unless replaced deliberately.
- **Success is now near-unused** — one comparison table. Worth deciding whether
  green belongs in the palette at all, or whether checks should be ink.
- **Dark mode is unexercised on the marketing site.** It is specified and should
  work, but nothing renders it, so regressions there are invisible.

---

## Related

`../` (brand-assets) holds the logo system — mark, wordmark, lockups, favicons,
built by `build_assets.py`. The design system and the logo share one palette and
both trace to the same brand pack. See `../CLAUDE.md`.
