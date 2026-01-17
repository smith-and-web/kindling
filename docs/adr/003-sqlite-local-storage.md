# ADR-003: Use SQLite for Local Data Storage

## Status

Accepted

## Context

Kindling needs persistent storage for project data (chapters, scenes, beats, characters, locations). The storage solution must:
- Work offline (local-first)
- Handle relational data (scenes belong to chapters, beats belong to scenes)
- Support efficient queries (loading scenes for a chapter)
- Be portable across platforms

Options considered:
- **File-based JSON**: Simple, human-readable, no query capability
- **SQLite**: Embedded relational database, single file, SQL queries
- **IndexedDB**: Browser-native, async, limited query capability
- **LevelDB/RocksDB**: Key-value stores, fast, no relations

## Decision

Use SQLite via the `rusqlite` crate in the Rust backend.

**Reasons:**
1. **Relational model fits the data**: Projects → Chapters → Scenes → Beats
2. **Single file**: Easy backup, portable, no server process
3. **Query capability**: SQL enables efficient filtering and ordering
4. **Rust integration**: `rusqlite` is mature and well-documented
5. **Transaction support**: Safe multi-table operations during import

## Consequences

### Positive
- Natural mapping from data model to tables
- Efficient queries with indexes
- ACID transactions for data integrity
- Single `.db` file is easy to backup/move
- Well-understood technology with extensive tooling

### Negative
- Schema migrations require careful handling
- No built-in sync capability (future cloud sync needs separate solution)
- Binary format not human-readable (debugging requires SQLite tools)

### Neutral
- Database lives in app data directory, separate from source files
- Need to handle database versioning for app updates
