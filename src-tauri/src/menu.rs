//! Native menus use the same bindings as the frontend command registry.
use std::collections::BTreeMap;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    AppHandle, Emitter, Manager, Wry,
};

const WINDOW_MINIMIZE: &str = "window_minimize";
const WINDOW_TOGGLE_MAXIMIZE: &str = "window_toggle_maximize";

/// Build a fresh menu: muda 0.17 on macOS does not clear the native key
/// equivalent when set_accelerator(None) is called on an existing item.
pub fn create_menu(app: &AppHandle<Wry>, bindings: &BTreeMap<String, String>) -> tauri::Result<()> {
    let command = |id: &str, label: &str| {
        let mut item = MenuItemBuilder::new(label).id(id);
        if let Some(binding) = bindings.get(id).filter(|value| !value.is_empty()) {
            item = item.accelerator(binding.replace("Mod+", "CmdOrCtrl+"));
        }
        item.build(app)
    };
    let import_submenu = SubmenuBuilder::new(app, "Import")
        .item(&command("import_plottr", "Plottr (.pltr)")?)
        .item(&command("import_ywriter", "yWriter 7 (.yw7)")?)
        .item(&command("import_markdown", "Markdown (.md)")?)
        .item(&command("import_longform", "Longform (Index or Vault...)")?)
        .item(&command("import_scrivener", "Scrivener 3 (.scriv)")?)
        .item(&command(
            "import_novelwriter",
            "novelWriter (Project Folder)",
        )?)
        .build()?;
    let file_submenu = SubmenuBuilder::new(app, "File")
        .item(&command("new_project", "New Project")?)
        .item(&command("editorial_open", "Open Review or Feedback File…")?)
        .item(&command("editorial_project", "Editorial Review…")?)
        .separator()
        .item(&import_submenu)
        .item(&command("export", "Export...")?)
        .separator()
        .item(&command("close_project", "Close Project")?)
        .separator()
        .item(&command("settings", "Settings...")?)
        .separator()
        .item(&command("quit", "Quit kindling")?)
        .build()?;
    let edit_submenu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .separator()
        .item(&command("find", "Find in Scene…")?)
        .item(&command("find_replace", "Find and Replace…")?)
        .item(&command("find_project", "Find and Replace in Project…")?)
        .build()?;
    // Close Project owns Mod+W; a second predefined Close Window item would
    // retain that accelerator after the user's Close Project binding changes.
    //
    // muda does not implement the predefined Minimize and Maximize items on
    // Linux, so the menu rendered empty there. Linux gets ordinary items that
    // setup_menu_events handles natively.
    let window_submenu = if cfg!(target_os = "linux") {
        SubmenuBuilder::new(app, "Window")
            .item(
                &MenuItemBuilder::new("Minimize")
                    .id(WINDOW_MINIMIZE)
                    .build(app)?,
            )
            .item(
                &MenuItemBuilder::new("Maximize")
                    .id(WINDOW_TOGGLE_MAXIMIZE)
                    .build(app)?,
            )
            .build()?
    } else {
        SubmenuBuilder::new(app, "Window")
            .minimize()
            .maximize()
            .build()?
    };
    let view_submenu = SubmenuBuilder::new(app, "View")
        .item(&command("toggle_sidebar", "Toggle Sidebar")?)
        .item(&command("toggle_references", "Toggle References Panel")?)
        .item(&command("sync", "Sync from Source")?)
        .build()?;
    let help_submenu = SubmenuBuilder::new(app, "Help")
        .item(&command("about", "About kindling…")?)
        .item(&command("send_feedback", "Send Feedback...")?)
        .separator()
        .item(&command("command_palette", "Command Palette...")?)
        .item(&command("quick_start", "Quick Start")?)
        .build()?;
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

pub fn setup_menu_events(app: &AppHandle<Wry>) {
    let app_handle = app.clone();
    app.on_menu_event(move |_app, event| {
        let Some(window) = app_handle.get_webview_window("main") else {
            return;
        };
        match event.id().0.as_str() {
            WINDOW_MINIMIZE => {
                let _ = window.minimize();
            }
            WINDOW_TOGGLE_MAXIMIZE => {
                if window.is_maximized().unwrap_or(false) {
                    let _ = window.unmaximize();
                } else {
                    let _ = window.maximize();
                }
            }
            id => {
                let _ = window.emit("menu-event", id);
            }
        }
    });
}
