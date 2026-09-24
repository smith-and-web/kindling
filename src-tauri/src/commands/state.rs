//! Application State
//!
//! Contains the global application state managed by Tauri.

use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::db::initialize_schema;

/// Records the app version that last opened the database, so an upgrade can be detected.
const VERSION_FILE: &str = "last-opened-version";
/// Pre-upgrade copies of `kindling.db`, newest `KEPT_BACKUPS` kept.
const BACKUP_DIR: &str = "backups";
const KEPT_BACKUPS: usize = 3;

/// Global application state managed by Tauri.
/// Contains the SQLite database connection wrapped in a Mutex for thread safety.
///
/// Commands use `conn.unchecked_transaction()` because `MutexGuard` yields
/// `&Connection` (not `&mut Connection`). Rollback is still automatic on drop;
/// the only difference from `transaction()` is that Rust won't prevent concurrent
/// `execute` calls at compile time (which the Mutex already prevents at runtime).
pub struct AppState {
    pub db: Mutex<Connection>,
}

impl AppState {
    pub fn new(app_data_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Self::open(&app_data_dir, env!("CARGO_PKG_VERSION"))
    }

    fn open(app_data_dir: &Path, app_version: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Ensure the data directory exists
        std::fs::create_dir_all(app_data_dir)?;

        let db_path = app_data_dir.join("kindling.db");
        let existing = std::fs::metadata(&db_path).is_ok_and(|m| m.len() > 0);
        let conn = Connection::open(&db_path)?;

        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        // Every project shares this one file, so copy it before a new version migrates it.
        let version_path = app_data_dir.join(VERSION_FILE);
        let last_version = std::fs::read_to_string(&version_path).ok();
        let last_version = last_version.as_deref().map(str::trim);
        let mut backup = None;
        if existing && last_version != Some(app_version) {
            match backup_database(&conn, app_data_dir, last_version.unwrap_or("unknown")) {
                Ok(path) => backup = Some(path),
                // A failed copy must not stop the app from opening; the migration is additive.
                Err(e) => {
                    eprintln!("[kindling] could not back up kindling.db before upgrading: {e}")
                }
            }
        }

        initialize_schema(&conn).map_err(|e| match &backup {
            Some(path) => format!(
                "Failed to prepare the kindling database: {e}. A copy made before this upgrade is at {}",
                path.display()
            ),
            None => format!("Failed to prepare the kindling database: {e}"),
        })?;
        if let Err(e) = std::fs::write(&version_path, app_version) {
            eprintln!("[kindling] could not record the database version: {e}");
        }

        Ok(Self {
            db: Mutex::new(conn),
        })
    }
}

/// Writes a consistent copy of the open database to `backups/` and prunes older copies.
fn backup_database(
    conn: &Connection,
    app_data_dir: &Path,
    from_version: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = app_data_dir.join(BACKUP_DIR);
    std::fs::create_dir_all(&dir)?;
    let version: String = from_version
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let path = dir.join(format!("kindling-{version}-{stamp}.db"));
    conn.execute("VACUUM INTO ?1", [path.to_string_lossy()])?;

    let mut backups: Vec<(std::time::SystemTime, PathBuf)> = std::fs::read_dir(&dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.starts_with("kindling-") && name.ends_with(".db")
        })
        .filter_map(|entry| Some((entry.metadata().ok()?.modified().ok()?, entry.path())))
        .collect();
    backups.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    for (_, old) in backups.into_iter().skip(KEPT_BACKUPS) {
        let _ = std::fs::remove_file(old);
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn backups(dir: &Path) -> Vec<PathBuf> {
        let Ok(entries) = std::fs::read_dir(dir.join(BACKUP_DIR)) else {
            return Vec::new();
        };
        let mut paths: Vec<PathBuf> = entries.map(|e| e.unwrap().path()).collect();
        paths.sort();
        paths
    }

    fn project_count(path: &Path) -> i64 {
        Connection::open(path)
            .unwrap()
            .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn fresh_install_makes_no_backup() {
        let dir = tempfile::tempdir().unwrap();
        AppState::open(dir.path(), "1.3.0").unwrap();
        assert!(backups(dir.path()).is_empty());
        assert_eq!(
            std::fs::read_to_string(dir.path().join(VERSION_FILE)).unwrap(),
            "1.3.0"
        );
    }

    #[test]
    fn upgrading_backs_up_the_database_once_per_version() {
        let dir = tempfile::tempdir().unwrap();
        {
            let state = AppState::open(dir.path(), "1.2.0").unwrap();
            let conn = state.db.lock().unwrap();
            conn.execute(
                "INSERT INTO projects (id, name, source_type, created_at, modified_at)
                 VALUES ('p', 'Novel', 'markdown', '2026-01-01', '2026-01-01')",
                [],
            )
            .unwrap();
        }
        assert!(backups(dir.path()).is_empty());

        AppState::open(dir.path(), "1.3.0").unwrap();
        let copies = backups(dir.path());
        assert_eq!(copies.len(), 1);
        let name = copies[0]
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert!(name.starts_with("kindling-1.2.0-"), "{name}");
        assert_eq!(project_count(&copies[0]), 1);

        AppState::open(dir.path(), "1.3.0").unwrap();
        assert_eq!(backups(dir.path()).len(), 1);
    }

    #[test]
    fn a_database_from_before_version_tracking_is_backed_up_and_old_copies_are_pruned() {
        let dir = tempfile::tempdir().unwrap();
        AppState::open(dir.path(), "1.2.0").unwrap();
        std::fs::remove_file(dir.path().join(VERSION_FILE)).unwrap();
        let backup_dir = dir.path().join(BACKUP_DIR);
        std::fs::create_dir_all(&backup_dir).unwrap();
        for i in 0..4 {
            std::fs::write(backup_dir.join(format!("kindling-old-{i}.db")), "x").unwrap();
        }
        std::fs::write(backup_dir.join("notes.txt"), "keep").unwrap();

        AppState::open(dir.path(), "1.3.0").unwrap();
        let copies = backups(dir.path());
        assert!(copies.iter().any(|p| p.ends_with("notes.txt")));
        let dbs: Vec<_> = copies
            .iter()
            .filter(|p| p.extension().is_some_and(|e| e == "db"))
            .collect();
        assert_eq!(dbs.len(), KEPT_BACKUPS);
        assert!(dbs
            .iter()
            .any(|p| p.to_string_lossy().contains("kindling-unknown-")));
    }
}
