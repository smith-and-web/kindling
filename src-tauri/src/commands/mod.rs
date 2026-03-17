//! Tauri IPC Command Handlers
//!
//! This module contains all the command handlers that the frontend can call
//! via Tauri's `invoke()` API. Commands are organized into submodules:
//!
//! - [`state`]: Application state management
//! - [`import`]: Import commands for Plottr, Markdown
//! - [`crud`]: CRUD operations for projects, chapters, scenes, beats
//! - [`sync`]: Sync/reimport functionality
//! - [`archive`]: Archive and restore commands
//! - [`lock`]: Lock/unlock commands
//! - [`export`]: Export commands for Markdown, DOCX
//! - [`snapshot`]: Snapshot/versioning commands
//! - [`settings`]: App-wide settings

mod archive;
mod blank_project;
mod crud;
mod detect;
mod export;
mod fields;
mod import;
mod lock;
mod sample_project;
mod settings;
mod snapshot;
mod state;
mod sync;
mod tags;

// Re-export everything for backwards compatibility with lib.rs
pub use archive::*;
pub use blank_project::*;
pub use crud::*;
pub use detect::*;
pub use export::*;
pub use fields::*;
pub use import::*;
pub use lock::*;
pub use sample_project::*;
pub use settings::*;
pub use snapshot::*;
pub use state::*;
pub use sync::*;
pub use tags::*;
