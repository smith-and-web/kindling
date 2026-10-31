//! Native menus use the same bindings as the frontend command registry.
use std::collections::BTreeMap;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    AppHandle, Emitter, Manager, Wry,
};

const WINDOW_MINIMIZE: &str = "window_minimize";
const WINDOW_TOGGLE_MAXIMIZE: &str = "window_toggle_maximize";

/// Native menu copy follows the system language, like the webview interface does.
fn use_chinese_menu() -> bool {
    tauri_plugin_os::locale()
        .map(|locale| locale.to_ascii_lowercase().starts_with("zh"))
        .unwrap_or(false)
}

/// Build a fresh menu: muda 0.17 on macOS does not clear the native key
/// equivalent when set_accelerator(None) is called on an existing item.
pub fn create_menu(app: &AppHandle<Wry>, bindings: &BTreeMap<String, String>) -> tauri::Result<()> {
    let chinese = use_chinese_menu();
    let l = |en: &'static str, zh: &'static str| if chinese { zh } else { en };
    let command = |id: &str, label: &str| {
        let mut item = MenuItemBuilder::new(label).id(id);
        if let Some(binding) = bindings.get(id).filter(|value| !value.is_empty()) {
            item = item.accelerator(binding.replace("Mod+", "CmdOrCtrl+"));
        }
        item.build(app)
    };
    let import_submenu = SubmenuBuilder::new(app, l("Import", "导入"))
        .item(&command("import_plottr", "Plottr (.pltr)")?)
        .item(&command("import_ywriter", "yWriter 7 (.yw7)")?)
        .item(&command("import_markdown", "Markdown (.md)")?)
        .item(&command(
            "import_longform",
            l("Longform (Index or Vault...)", "Longform（索引或 Vault…）"),
        )?)
        .item(&command("import_scrivener", "Scrivener 3 (.scriv)")?)
        .item(&command(
            "import_novelwriter",
            l("novelWriter (Project Folder)", "novelWriter（项目文件夹）"),
        )?)
        .build()?;
    let file_submenu = SubmenuBuilder::new(app, l("File", "文件"))
        .item(&command("new_project", l("New Project", "新建项目"))?)
        .item(&command(
            "editorial_open",
            l("Open Review or Feedback File…", "打开审阅或反馈文件…"),
        )?)
        .item(&command(
            "editorial_project",
            l("Editorial Review…", "审阅…"),
        )?)
        .separator()
        .item(&import_submenu)
        .item(&command("export", l("Export...", "导出…"))?)
        .separator()
        .item(&command("close_project", l("Close Project", "关闭项目"))?)
        .separator()
        .item(&command("settings", l("Settings...", "设置…"))?)
        .separator()
        .item(&command("quit", l("Quit kindling", "退出 kindling"))?)
        .build()?;
    let edit_submenu = SubmenuBuilder::new(app, l("Edit", "编辑"))
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .separator()
        .item(&command("find", l("Find in Scene…", "在场景中查找…"))?)
        .item(&command(
            "find_replace",
            l("Find and Replace…", "查找和替换…"),
        )?)
        .item(&command(
            "find_project",
            l("Find and Replace in Project…", "在项目中查找和替换…"),
        )?)
        .build()?;
    // Close Project owns Mod+W; a second predefined Close Window item would
    // retain that accelerator after the user's Close Project binding changes.
    //
    // muda does not implement the predefined Minimize and Maximize items on
    // Linux, so the menu rendered empty there. Linux gets ordinary items that
    // setup_menu_events handles natively.
    let window_submenu = if cfg!(target_os = "linux") {
        SubmenuBuilder::new(app, l("Window", "窗口"))
            .item(
                &MenuItemBuilder::new(l("Minimize", "最小化"))
                    .id(WINDOW_MINIMIZE)
                    .build(app)?,
            )
            .item(
                &MenuItemBuilder::new(l("Maximize", "最大化"))
                    .id(WINDOW_TOGGLE_MAXIMIZE)
                    .build(app)?,
            )
            .build()?
    } else {
        SubmenuBuilder::new(app, l("Window", "窗口"))
            .minimize()
            .maximize()
            .build()?
    };
    let view_submenu = SubmenuBuilder::new(app, l("View", "视图"))
        .item(&command(
            "toggle_sidebar",
            l("Toggle Sidebar", "显示/隐藏侧边栏"),
        )?)
        .item(&command(
            "toggle_references",
            l("Toggle References Panel", "显示/隐藏引用面板"),
        )?)
        .item(&command("sync", l("Sync from Source", "从源文件同步"))?)
        .build()?;
    let help_submenu = SubmenuBuilder::new(app, l("Help", "帮助"))
        .item(&command("about", l("About kindling…", "关于 kindling…"))?)
        .item(&command(
            "send_feedback",
            l("Send Feedback...", "发送反馈…"),
        )?)
        .separator()
        .item(&command(
            "command_palette",
            l("Command Palette...", "命令面板…"),
        )?)
        .item(&command("quick_start", l("Quick Start", "快速入门"))?)
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
