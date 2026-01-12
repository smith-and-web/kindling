# Kindling

**Spark your draft** — Bridge the gap between outline and prose.

Kindling is a desktop writing application that helps fiction writers overcome the blank page problem when moving from outline to first draft. Import your story structure from popular outlining tools, then write prose directly into a scaffolded view where scene beats appear as expandable prompts.

## The Problem

Writers often struggle with the transition from outline to draft. You've done the planning work—you know your story beats, your character arcs, your scene goals—but when you open a blank document, that carefully crafted structure feels distant and unhelpful.

## The Solution

Kindling keeps your outline visible and actionable as you write. Each scene displays its beats as collapsible cards that you can expand to write prose directly beneath them. Your outline becomes the scaffolding for your draft, always present but never in the way.

## Features

- **Import from popular tools**: Plottr (.pltr), Scrivener (.scriv), or Markdown outlines
- **Scaffolded writing view**: Scene beats appear as expandable prompts
- **Distraction-free dark mode**: Easy on the eyes for long writing sessions
- **Local-first**: Your work stays on your machine in a SQLite database
- **Cross-platform**: macOS and Windows support

## Tech Stack

- **Frontend**: Svelte 5 with Tailwind CSS
- **Backend**: Rust with Tauri 2.x
- **Database**: SQLite via rusqlite
- **Parsers**: Native Rust parsers for Plottr, Scrivener, and Markdown

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) (latest stable)
- Platform-specific dependencies for Tauri: [see Tauri prerequisites](https://tauri.app/start/prerequisites/)

### Setup

```bash
# Clone the repository
git clone https://github.com/smith-and-web/kindling.git
cd kindling

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Project Structure

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

## Roadmap

### Phase 1: Scaffold ✅
- Project initialization with Tauri 2.x + Svelte 5
- SQLite database schema
- Rust data models
- Basic UI structure

### Phase 2: Import (In Progress)
- Plottr parser implementation
- Scrivener parser implementation
- Markdown outline parser
- Project selection UI

### Phase 3: Editor
- Beat-by-beat writing interface
- Prose persistence
- Word count tracking
- Auto-save

### Phase 4: Polish
- Export to Markdown/DOCX
- Character/Location reference panels
- Keyboard shortcuts
- Theme customization

## Contributing

Contributions are welcome! This project is in early development, so there's plenty of opportunity to shape its direction.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Built with [Tauri](https://tauri.app/) and [Svelte](https://svelte.dev/)
- Inspired by the writing workflows of [Plottr](https://plottr.com/) and [Scrivener](https://www.literatureandlatte.com/scrivener/)
