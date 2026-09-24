# README demo recorder

`npm run demo:readme` rebuilds `docs/assets/kindling-demo.svg`, the animated demo at
the top of the project README. It drives a real kindling session: a hidden,
isolated debug build controlled through its authenticated Unix socket, the same
transport as [visual QA](../../visual/README.md). The output is vector, not video.

## How it works

1. `record.mjs` builds the debug binary and starts it hidden with
   `KINDLING_QA_BACKGROUND=1` and a fresh `KINDLING_DATA_DIR` in the system temp
   directory. It then sets a 1280×800 viewport.
2. It drives the app the way a user would: it clicks **Sample project**, opens
   _On the Cliff_, expands a beat, and types prose into the editor. It opens Find
   and Replace, editorial review and the export workspace with their real keyboard
   shortcuts, then switches to the dark theme in Settings.
3. At each checkpoint, `dom2svg.js` runs inside the webview and serialises the
   rendered document into SVG from computed layout and styles. Serialised
   properties include backgrounds, borders, shadows, gradients, Lucide icons,
   images, form values, ellipses and line clamps, modal dialogs, and the
   sandboxed export-preview iframe. Text is kept as positioned words.
4. The frames are combined into one looping SMIL animation with captions, a
   pointer, and word-by-word typing over the final typed state. Inter, Fraunces
   and Newsreader are subset to the glyphs used and embedded as WOFF2, so the SVG
   renders the same in GitHub's `<img>` sandbox.

The app's WebKit `localStorage` is shared with the visual QA profile. The recorder
snapshots it before starting and restores it exactly afterwards. The scratch
library is deleted when the run ends.

## Run

Prerequisites: macOS 14+, this checkout's Node and Rust dependencies, and
[fonttools](https://github.com/fonttools/fonttools) with Brotli support for
`pyftsubset`. A scratch virtual environment keeps fonttools out of your system Python:

```bash
python3 -m venv /tmp/fonttools && /tmp/fonttools/bin/pip install fonttools brotli
```

Start Vite for this checkout (`npm run dev` or `npm run tauri dev`), make sure no
visual QA app is serving `/tmp/kindling-qa.sock`, then run:

```bash
PYFTSUBSET=/tmp/fonttools/bin/pyftsubset npm run demo:readme
```

Options:

- `-- --png` also saves each checkpoint's WKWebView snapshot to
  `qa/demo/readme/results/` (gitignored), so you can compare them with the SVG.
- `-- --keep-app` leaves the hidden app and its scratch library running for
  inspection.

A run takes about a minute and writes a file of roughly 1 MB (about 400 KB
gzipped). Don't edit files in the checkout while it runs: Vite reloads the page,
which invalidates the session.

## Changing the story

Scenes, their durations and captions are defined in `record()` in `record.mjs`.
Captions marked `badge: true` get a **New in 1.3** pill; update that label in
`assemble()` for the next release. Keep captions to what the frame actually shows.
When you add a UI pattern that the serialiser doesn't know yet, use `--png` and
compare the frame against the snapshot before committing.
