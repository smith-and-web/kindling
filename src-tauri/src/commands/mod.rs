//! Tauri IPC Command Handlers
//!
//! This module contains all the command handlers that the frontend can call
//! via Tauri's `invoke()` API. Commands are organized into submodules:
//!
//! - [`state`]: Application state management
//! - [`import`]: Import commands for Plottr, Scrivener, Markdown
//! - [`crud`]: CRUD operations for projects, chapters, scenes, beats
//! - [`sync`]: Sync/reimport functionality
//! - [`archive`]: Archive and restore commands
//! - [`lock`]: Lock/unlock commands
//! - [`export`]: Export commands for Markdown, DOCX

mod archive;
mod crud;
mod export;
mod import;
mod lock;
mod state;
mod sync;

// Re-export everything for backwards compatibility with lib.rs
pub use archive::*;
pub use crud::*;
pub use export::*;
pub use import::*;
pub use lock::*;
pub use state::*;
pub use sync::*;
