//! Application Menu Setup
//!
//! Creates the native application menu with File menu items for:
//! - Import (Plottr, Markdown)
//! - Import (Longform)
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
    pub const NEW_PROJECT: &str = "new_project";
    pub const IMPORT_PLOTTR: &str = "import_plottr";
    pub const IMPORT_YWRITER: &str = "import_ywriter";
    pub const IMPORT_MARKDOWN: &str = "import_markdown";
    pub const IMPORT_LONGFORM: &str = "import_longform";
    pub const EXPORT: &str = "export";
    pub const CLOSE_PROJECT: &str = "close_project";
    pub const PROJECT_SETTINGS: &str = "project_settings";
    pub const KINDLING_SETTINGS: &str = "kindling_settings";
    pub const QUICK_START: &str = "quick_start";
    pub const TOGGLE_SIDEBAR: &str = "toggle_sidebar";
    pub const TOGGLE_REFERENCES: &str = "toggle_references";
    pub const SYNC: &str = "sync";
    pub const COMMAND_PALETTE: &str = "command_palette";
    pub const ABOUT: &str = "about";
    pub const QUIT: &str = "quit";
}

/// Create the application menu
pub fn create_menu(app: &AppHandle<Wry>) -> Result<(), Box<dyn std::error::Error>> {
    // Import submenu
    let import_plottr = MenuItemBuilder::new("Plottr (.pltr)")
        .id(menu_ids::IMPORT_PLOTTR)
        .accelerator("CmdOrCtrl+Shift+O")
        .build(app)?;

    let import_ywriter = MenuItemBuilder::new("yWriter 7 (.yw7)")
        .id(menu_ids::IMPORT_YWRITER)
        .accelerator("CmdOrCtrl+Shift+Y")
        .build(app)?;

    let import_markdown = MenuItemBuilder::new("Markdown (.md)")
        .id(menu_ids::IMPORT_MARKDOWN)
        .accelerator("CmdOrCtrl+Shift+M")
        .build(app)?;

    let import_longform = MenuItemBuilder::new("Longform (Index or Vault...)")
        .id(menu_ids::IMPORT_LONGFORM)
        .accelerator("CmdOrCtrl+Shift+L")
        .build(app)?;

    let import_submenu = SubmenuBuilder::new(app, "Import")
        .item(&import_plottr)
        .item(&import_ywriter)
        .item(&import_markdown)
        .item(&import_longform)
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
        .accelerator("CmdOrCtrl+Shift+P")
        .build(app)?;

    let kindling_settings = MenuItemBuilder::new("Kindling Settings...")
        .id(menu_ids::KINDLING_SETTINGS)
        .accelerator("CmdOrCtrl+,")
        .build(app)?;

    let new_project = MenuItemBuilder::new("New Project")
        .id(menu_ids::NEW_PROJECT)
        .accelerator("CmdOrCtrl+N")
        .build(app)?;

    let quit = MenuItemBuilder::new("Quit Kindling")
        .id(menu_ids::QUIT)
        .accelerator("CmdOrCtrl+Q")
        .build(app)?;

    // Build File submenu
    let file_submenu = SubmenuBuilder::new(app, "File")
        .item(&new_project)
        .separator()
        .items(&[&import_submenu])
        .item(&export)
        .separator()
        .item(&close_project)
        .separator()
        .item(&project_settings)
        .item(&kindling_settings)
        .separator()
        .item(&quit)
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

    // View submenu
    let toggle_sidebar = MenuItemBuilder::new("Toggle Sidebar")
        .id(menu_ids::TOGGLE_SIDEBAR)
        .accelerator("CmdOrCtrl+Backslash")
        .build(app)?;

    let toggle_references = MenuItemBuilder::new("Toggle References Panel")
        .id(menu_ids::TOGGLE_REFERENCES)
        .accelerator("CmdOrCtrl+Shift+R")
        .build(app)?;

    let sync = MenuItemBuilder::new("Sync from Source")
        .id(menu_ids::SYNC)
        .accelerator("CmdOrCtrl+Shift+S")
        .build(app)?;

    let view_submenu = SubmenuBuilder::new(app, "View")
        .item(&toggle_sidebar)
        .item(&toggle_references)
        .item(&sync)
        .build()?;

    // Help submenu
    let about = MenuItemBuilder::new("About Kindling...")
        .id(menu_ids::ABOUT)
        .build(app)?;

    let command_palette = MenuItemBuilder::new("Command Palette...")
        .id(menu_ids::COMMAND_PALETTE)
        .accelerator("CmdOrCtrl+K")
        .build(app)?;

    let quick_start = MenuItemBuilder::new("Quick Start")
        .id(menu_ids::QUICK_START)
        .accelerator("CmdOrCtrl+Shift+H")
        .build(app)?;

    let help_submenu = SubmenuBuilder::new(app, "Help")
        .item(&about)
        .separator()
        .item(&command_palette)
        .item(&quick_start)
        .build()?;

    // Build the full menu
    let menu = MenuBuilder::new(app)
        .items(&[
            &file_submenu,
            &edit_submenu,
            &view_submenu,
            &window_submenu,
            &help_submenu,
        ])
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
