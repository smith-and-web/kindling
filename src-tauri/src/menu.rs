//! Application Menu Setup
//!
//! Creates the native application menu with File menu items for:
//! - Import (Plottr, Scrivener, Markdown)
//! - Export
//! - Close Project
//! - Project Settings
//! - Kindling Settings

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    AppHandle, Emitter, Manager, Wry,
};

/// Menu item IDs for event handling
pub mod menu_ids {
    pub const IMPORT_PLOTTR: &str = "import_plottr";
    pub const IMPORT_SCRIVENER: &str = "import_scrivener";
    pub const IMPORT_MARKDOWN: &str = "import_markdown";
    pub const EXPORT: &str = "export";
    pub const CLOSE_PROJECT: &str = "close_project";
    pub const PROJECT_SETTINGS: &str = "project_settings";
    pub const KINDLING_SETTINGS: &str = "kindling_settings";
}

/// Create the application menu
pub fn create_menu(app: &AppHandle<Wry>) -> Result<(), Box<dyn std::error::Error>> {
    // Import submenu
    let import_plottr = MenuItemBuilder::new("Plottr (.pltr)")
        .id(menu_ids::IMPORT_PLOTTR)
        .build(app)?;

    let import_scrivener = MenuItemBuilder::new("Scrivener (.scriv)")
        .id(menu_ids::IMPORT_SCRIVENER)
        .build(app)?;

    let import_markdown = MenuItemBuilder::new("Markdown (.md)")
        .id(menu_ids::IMPORT_MARKDOWN)
        .build(app)?;

    let import_submenu = SubmenuBuilder::new(app, "Import")
        .item(&import_plottr)
        .item(&import_scrivener)
        .item(&import_markdown)
        .build()?;

    // Export menu item
    let export = MenuItemBuilder::new("Export...")
        .id(menu_ids::EXPORT)
        .accelerator("CmdOrCtrl+E")
        .build(app)?;

    // Close Project menu item
    let close_project = MenuItemBuilder::new("Close Project")
        .id(menu_ids::CLOSE_PROJECT)
        .accelerator("CmdOrCtrl+W")
        .build(app)?;

    // Settings menu items
    let project_settings = MenuItemBuilder::new("Project Settings...")
        .id(menu_ids::PROJECT_SETTINGS)
        .build(app)?;

    let kindling_settings = MenuItemBuilder::new("Kindling Settings...")
        .id(menu_ids::KINDLING_SETTINGS)
        .accelerator("CmdOrCtrl+,")
        .build(app)?;

    // Build File submenu
    let file_submenu = SubmenuBuilder::new(app, "File")
        .items(&[&import_submenu])
        .item(&export)
        .separator()
        .item(&close_project)
        .separator()
        .item(&project_settings)
        .item(&kindling_settings)
        .build()?;

    // Build Edit submenu with standard items
    let edit_submenu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    // Build Window submenu with standard items
    let window_submenu = SubmenuBuilder::new(app, "Window")
        .minimize()
        .maximize()
        .separator()
        .close_window()
        .build()?;

    // Build the full menu
    let menu = MenuBuilder::new(app)
        .items(&[&file_submenu, &edit_submenu, &window_submenu])
        .build()?;

    app.set_menu(menu)?;

    Ok(())
}

/// Set up menu event handling
pub fn setup_menu_events(app: &AppHandle<Wry>) {
    let app_handle = app.clone();

    app.on_menu_event(move |_app, event| {
        let id = event.id().0.as_str();

        // Emit event to frontend for handling
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.emit("menu-event", id);
        }
    });
}
