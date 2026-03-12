<p align="center">
  <img src=".github/assets/kindling-mark-color-512.png" alt="Kindling" width="128" height="128" />
</p>

<h1 align="center">Kindling</h1>

<p align="center">
  <strong>Free, open-source writing software for plotters and outliners.</strong><br/>
  Bridge the gap between your story outline and your first draft.
</p>

<p align="center">
  <a href="https://kindlingwriter.com/download/">Download</a> ·
  <a href="https://kindlingwriter.com/features/">Features</a> ·
  <a href="https://kindlingwriter.com/compare/">Compare</a> ·
  <a href="#contributing">Contributing</a>
</p>

<p align="center">
  <a href="https://github.com/smith-and-web/kindling/actions/workflows/ci.yml">
    <img src="https://github.com/smith-and-web/kindling/actions/workflows/ci.yml/badge.svg" alt="CI Status" />
  </a>
  <a href="https://github.com/smith-and-web/kindling/releases">
    <img src="https://img.shields.io/github/v/release/smith-and-web/kindling?include_prereleases&label=version" alt="Version" />
  </a>
  <a href="https://github.com/smith-and-web/kindling/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/smith-and-web/kindling" alt="License" />
  </a>
  <a href="https://github.com/smith-and-web/kindling/stargazers">
    <img src="https://img.shields.io/github/stars/smith-and-web/kindling?style=flat" alt="Stars" />
  </a>
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey" alt="Platform" />
</p>

---

<p align="center">
  <img src="docs/assets/demo-1.gif" alt="Kindling demo - scaffolded writing view" width="800" />
</p>

## Why Kindling?

- **Your outline stays visible while you write.** Scene beats appear as expandable prompts in your drafting space. No more switching between apps.
- **Import your existing work.** Bring in projects from Plottr (.pltr), yWriter (.yw7), or Obsidian Longform — no starting from scratch.
- **No AI. No subscription. No cloud.** Every word is yours. Your projects are local SQLite files. Works completely offline.
- **Free and open source.** MIT licensed. Inspect the code, contribute, or fork it. Your tools should be as permanent as your writing.

## Download

Get Kindling for free at **[kindlingwriter.com/download](https://kindlingwriter.com/download/)**

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) | `Kindling_*_aarch64.dmg` |
| macOS (Intel) | `Kindling_*_x64.dmg` |
| Windows | `Kindling_*_x64-setup.msi` |
| Linux | `Kindling_*_amd64.AppImage` or `.deb` |

Or grab the latest directly from the [Releases page](https://github.com/smith-and-web/kindling/releases).

## Features

| Feature | Description |
|---------|-------------|
| **Import from popular tools** | Plottr (`.pltr`), Markdown (`.md`), yWriter (`.yw7`), and Longform/Obsidian (index or vault) |
| **Scaffolded writing view** | Scene beats appear as expandable prompts |
| **Rich text prose editor** | Write with formatting, auto-save, and beat context |
| **Export formats** | DOCX (Standard Manuscript Format), Markdown, Longform/Obsidian, and EPUB |
| **Reference types** | Characters, locations, items, objectives, and organizations |
| **Sync/reimport** | Preview and apply source changes while preserving prose |
| **Distraction-free dark mode** | Easy on the eyes for long writing sessions |
| **Local-first** | Your work stays on your machine in a SQLite database |
| **Cross-platform** | macOS, Windows, and Linux support |

See the full [features overview](https://kindlingwriter.com/features/) on the website.

## Tech Stack

- **Frontend**: [Svelte 5](https://svelte.dev/) + [Tailwind CSS](https://tailwindcss.com/)
- **Backend**: [Rust](https://www.rust-lang.org/) + [Tauri 2.x](https://tauri.app/)
- **Database**: [SQLite](https://sqlite.org/) via rusqlite
- **Parsers**: Native Rust parsers for Plottr and Markdown

## From Source

**Prerequisites:**
- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) (stable)
- Platform dependencies: [Tauri prerequisites](https://tauri.app/start/prerequisites/)

```bash
# Clone the repository
git clone https://github.com/smith-and-web/kindling.git
cd kindling

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Roadmap

Track progress on the [project board](https://github.com/users/smith-and-web/projects/1).

| Phase | Status | Description |
|-------|--------|-------------|
| **v0.1 - Foundation** | ✅ Complete | Plottr import, basic UI, project structure |
| **v0.2 - Outline View** | ✅ Complete | Drag-and-drop reordering, create/delete scenes |
| **v0.3 - Writing & Export** | ✅ Complete | Prose editor, DOCX export with Standard Manuscript Format |
| **v1.0 - Release** | ✅ Complete | Additional importers, polish, performance, stability |
| **v1.2 - Next** | 🔄 In Progress | Sync/reimport UI, bug fixes, new features |

See the [milestones](https://github.com/smith-and-web/kindling/milestones) for detailed breakdowns.

## Testing

Kindling maintains high test coverage standards to ensure code quality and prevent regressions.

| Metric | Minimum | Current |
|--------|---------|---------|
| Statements | 95% | 95%+ |
| Branches | 65% | 65%+ |
| Functions | 98% | 98%+ |
| Lines | 95% | 95%+ |

**CI will fail if coverage drops below these thresholds.** New code must include appropriate tests.

```bash
# Frontend tests with coverage
npm test -- --coverage

# Rust tests
cd src-tauri && cargo test

# Run all checks (lint, format, types, tests)
npm run check:all
```

## Contributing

Contributions are welcome! Please read the [Contributing Guide](CONTRIBUTING.md) before submitting a PR.

- 🐛 [Report bugs](https://github.com/smith-and-web/kindling/issues/new?template=bug_report.yml)
- 💡 [Request features](https://github.com/smith-and-web/kindling/issues/new?template=feature_request.yml)
- 💬 [GitHub Discussions](https://github.com/smith-and-web/kindling/discussions) — Questions and ideas
- 🔥 [Discord](https://discord.gg/g7bkj4kY8w) — Chat with other writers and contributors

Looking for a place to start? Check out issues labeled [`good first issue`](https://github.com/smith-and-web/kindling/labels/good%20first%20issue).

## Support

If Kindling is useful to you, consider supporting its development:

<a href="https://github.com/sponsors/smith-and-web">
  <img src="https://img.shields.io/badge/Sponsor-❤️-ea4aaa?style=for-the-badge&logo=github-sponsors" alt="Sponsor on GitHub" />
</a>

Your sponsorship helps keep Kindling free and open source.

## License

[MIT](LICENSE) — free for personal and commercial use.

## Acknowledgments

- Built with [Tauri](https://tauri.app/) and [Svelte](https://svelte.dev/)
- Inspired by [Plottr](https://plottr.com/) and [Scrivener](https://www.literatureandlatte.com/scrivener/)

---

<p align="center">
  Made with ☕ for writers who plan before they write.
</p>
