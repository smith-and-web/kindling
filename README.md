<p align="center">
  <img src=".github/assets/kindling-mark-color-512.png" alt="Kindling" width="128" height="128" />
</p>

<h1 align="center">Kindling</h1>

<p align="center">
  <strong>Spark your draft</strong> — Bridge the gap between outline and prose.
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
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#roadmap">Roadmap</a> •
  <a href="#contributing">Contributing</a> •
  <a href="#support">Support</a>
</p>

---

## The Problem

Writers often struggle with the transition from outline to draft. You've done the planning work—you know your story beats, your character arcs, your scene goals—but when you open a blank document, that carefully crafted structure feels distant and unhelpful.

## The Solution

Kindling keeps your outline visible and actionable as you write. Each scene displays its beats as collapsible cards that you can expand to write prose directly beneath them. Your outline becomes the scaffolding for your draft, always present but never in the way.

## Features

| Feature | Description |
|---------|-------------|
| **Import from popular tools** | Plottr (.pltr), Scrivener (.scriv), or Markdown outlines |
| **Scaffolded writing view** | Scene beats appear as expandable prompts |
| **Distraction-free dark mode** | Easy on the eyes for long writing sessions |
| **Local-first** | Your work stays on your machine in a SQLite database |
| **Cross-platform** | macOS, Windows, and Linux support |

## Installation

> **Note**: Kindling is in early development. Pre-built releases are coming soon.

### From Source

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
| **v0.1 - Foundation** | 🔄 In Progress | Plottr import, basic UI, project structure |
| **v0.2 - Outline View** | ⏳ Planned | Drag-and-drop reordering, create/delete scenes |
| **v0.3 - Writing** | ⏳ Planned | Prose editor with beat integration |
| **v0.4 - Export** | ⏳ Planned | Export to Scrivener, DOCX, Markdown |
| **v1.0 - Release** | ⏳ Planned | Polish, performance, stability |

See the [milestones](https://github.com/smith-and-web/kindling/milestones) for detailed breakdowns.

## Tech Stack

- **Frontend**: [Svelte 5](https://svelte.dev/) + [Tailwind CSS](https://tailwindcss.com/)
- **Backend**: [Rust](https://www.rust-lang.org/) + [Tauri 2.x](https://tauri.app/)
- **Database**: [SQLite](https://sqlite.org/) via rusqlite
- **Parsers**: Native Rust parsers for Plottr, Scrivener, and Markdown

<details>
<summary><strong>Project Structure</strong></summary>

```
kindling/
├── src/                          # Svelte frontend
│   ├── lib/
│   │   ├── components/           # UI components
│   │   ├── stores/               # Svelte 5 state management
│   │   └── types.ts              # TypeScript interfaces
│   ├── app.css                   # Tailwind styles
│   └── App.svelte                # Main app component
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── commands.rs           # Tauri IPC handlers
│   │   ├── db/                   # SQLite schema & queries
│   │   ├── models/               # Data structures
│   │   └── parsers/              # Import parsers
│   └── tauri.conf.json           # Tauri configuration
└── package.json
```

</details>

## Contributing

Contributions are welcome! Please read the [Contributing Guide](CONTRIBUTING.md) before submitting a PR.

- 🐛 [Report bugs](https://github.com/smith-and-web/kindling/issues/new?template=bug_report.yml)
- 💡 [Request features](https://github.com/smith-and-web/kindling/issues/new?template=feature_request.yml)
- 💬 [Join discussions](https://github.com/smith-and-web/kindling/discussions)

Looking for a place to start? Check out issues labeled [`good first issue`](https://github.com/smith-and-web/kindling/labels/good%20first%20issue).

## Support

If Kindling is useful to you, consider supporting its development:

<a href="https://github.com/sponsors/smith-and-web">
  <img src="https://img.shields.io/badge/Sponsor-❤️-ea4aaa?style=for-the-badge&logo=github-sponsors" alt="Sponsor on GitHub" />
</a>

Your sponsorship helps keep Kindling free and open source. See the [sponsor tiers](https://github.com/sponsors/smith-and-web) for perks.

## License

[MIT](LICENSE) — free for personal and commercial use.

## Acknowledgments

- Built with [Tauri](https://tauri.app/) and [Svelte](https://svelte.dev/)
- Inspired by [Plottr](https://plottr.com/) and [Scrivener](https://www.literatureandlatte.com/scrivener/)

---

<p align="center">
  Made with ☕ for writers who plan before they write.
</p>
