//! Application State
//!
//! Contains the global application state managed by Tauri.

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::db::initialize_schema;

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
        // Ensure the data directory exists
        std::fs::create_dir_all(&app_data_dir)?;

        let db_path = app_data_dir.join("kindling.db");
        let conn = Connection::open(&db_path)?;

        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        initialize_schema(&conn)?;

        Ok(Self {
            db: Mutex::new(conn),
        })
    }
}
