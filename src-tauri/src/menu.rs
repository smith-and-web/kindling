//! Native menus use the same bindings as the frontend command registry.
use std::collections::BTreeMap;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    AppHandle, Emitter, Manager, Wry,
};

const WINDOW_MINIMIZE: &str = "window_minimize";
const WINDOW_TOGGLE_MAXIMIZE: &str = "window_toggle_maximize";

/// Native menu copy for one interface locale. Every field is required, so a locale
/// missing a label does not compile. Import format names (`Plottr (.pltr)`) and the
/// predefined Edit items AppKit draws itself are not translated here.
pub struct MenuStrings {
    pub file: &'static str,
    pub new_project: &'static str,
    pub editorial_open: &'static str,
    pub editorial_project: &'static str,
    pub import: &'static str,
    pub import_longform: &'static str,
    pub import_novelwriter: &'static str,
    pub export: &'static str,
    pub close_project: &'static str,
    pub settings: &'static str,
    pub quit: &'static str,
    pub edit: &'static str,
    pub find: &'static str,
    pub find_replace: &'static str,
    pub find_project: &'static str,
    pub view: &'static str,
    pub toggle_sidebar: &'static str,
    pub toggle_references: &'static str,
    pub sync: &'static str,
    pub window: &'static str,
    pub minimize: &'static str,
    pub maximize: &'static str,
    pub help: &'static str,
    pub about: &'static str,
    pub send_feedback: &'static str,
    pub command_palette: &'static str,
    pub quick_start: &'static str,
}

pub const EN: MenuStrings = MenuStrings {
    file: "File",
    new_project: "New Project",
    editorial_open: "Open Review or Feedback File…",
    editorial_project: "Editorial Review…",
    import: "Import",
    import_longform: "Longform (Index or Vault...)",
    import_novelwriter: "novelWriter (Project Folder)",
    export: "Export...",
    close_project: "Close Project",
    settings: "Settings...",
    quit: "Quit kindling",
    edit: "Edit",
    find: "Find in Scene…",
    find_replace: "Find and Replace…",
    find_project: "Find and Replace in Project…",
    view: "View",
    toggle_sidebar: "Toggle Sidebar",
    toggle_references: "Toggle References Panel",
    sync: "Sync from Source",
    window: "Window",
    minimize: "Minimize",
    maximize: "Maximize",
    help: "Help",
    about: "About kindling…",
    send_feedback: "Send Feedback...",
    command_palette: "Command Palette...",
    quick_start: "Quick Start",
};

pub const ZH_CN: MenuStrings = MenuStrings {
    file: "文件",
    new_project: "新建项目",
    editorial_open: "打开审阅或反馈文件…",
    editorial_project: "审阅…",
    import: "导入",
    import_longform: "Longform（索引或 Vault…）",
    import_novelwriter: "novelWriter（项目文件夹）",
    export: "导出…",
    close_project: "关闭项目",
    settings: "设置…",
    quit: "退出 kindling",
    edit: "编辑",
    find: "在场景中查找…",
    find_replace: "查找和替换…",
    find_project: "在项目中查找和替换…",
    view: "视图",
    toggle_sidebar: "显示/隐藏侧边栏",
    toggle_references: "显示/隐藏引用面板",
    sync: "从源文件同步",
    window: "窗口",
    minimize: "最小化",
    maximize: "最大化",
    help: "帮助",
    about: "关于 kindling…",
    send_feedback: "发送反馈…",
    command_palette: "命令面板…",
    quick_start: "快速入门",
};

pub const ES: MenuStrings = MenuStrings {
    file: "Archivo",
    new_project: "Nuevo proyecto",
    editorial_open: "Abrir archivo de revisión o feedback…",
    editorial_project: "Revisión editorial…",
    import: "Importar",
    import_longform: "Longform (índice o Vault…)",
    import_novelwriter: "novelWriter (carpeta del proyecto)",
    export: "Exportar…",
    close_project: "Cerrar proyecto",
    settings: "Ajustes…",
    quit: "Salir de kindling",
    edit: "Edición",
    find: "Buscar en la escena…",
    find_replace: "Buscar y reemplazar…",
    find_project: "Buscar y reemplazar en el proyecto…",
    view: "Visualización",
    toggle_sidebar: "Mostrar u ocultar la barra lateral",
    toggle_references: "Mostrar u ocultar el panel de referencias",
    sync: "Sincronizar desde la fuente",
    window: "Ventana",
    minimize: "Minimizar",
    maximize: "Maximizar",
    help: "Ayuda",
    about: "Acerca de kindling…",
    send_feedback: "Enviar feedback…",
    command_palette: "Paleta de comandos…",
    quick_start: "Inicio rápido",
};

/// Supported locales: the catalog directory name, the system language subtags it serves
/// and its menu copy. Mirrors `LOCALES` in `src/lib/i18n/index.ts`;
/// `src/lib/i18n/detection.json` holds the cases both test suites check.
const LOCALES: &[(&str, &[&str], &MenuStrings)] = &[
    ("en", &["en"], &EN),
    ("zh-CN", &["zh"], &ZH_CN),
    ("es", &["es"], &ES),
];

const FALLBACK_LOCALE: &str = "en";

/// Map a BCP 47 tag such as `zh-Hant-TW` or `en_GB` to the supported locale that serves it.
/// An exact locale (`pt-BR`) wins over a language match (`pt`); anything else is English.
pub fn resolve_locale(tag: &str) -> &'static str {
    let wanted = tag.trim().replace('_', "-").to_ascii_lowercase();
    if let Some((locale, _, _)) = LOCALES
        .iter()
        .find(|(locale, _, _)| locale.to_ascii_lowercase() == wanted)
    {
        return locale;
    }
    let language = wanted.split('-').next().unwrap_or_default();
    LOCALES
        .iter()
        .find(|(_, languages, _)| languages.contains(&language))
        .map_or(FALLBACK_LOCALE, |(locale, _, _)| locale)
}

/// `KINDLING_LOCALE` wins over the system language, so QA can check every locale on one
/// machine. It also reaches the webview through `vite.config.ts` under `npm run tauri dev`.
fn pick_locale(override_tag: Option<String>, system_tag: Option<String>) -> &'static str {
    let tag = override_tag
        .filter(|tag| !tag.trim().is_empty())
        .or(system_tag)
        .unwrap_or_default();
    resolve_locale(&tag)
}

/// Native menu copy follows the system language, like the webview interface does.
fn menu_locale() -> &'static str {
    pick_locale(
        std::env::var("KINDLING_LOCALE").ok(),
        tauri_plugin_os::locale(),
    )
}

pub fn menu_strings(locale: &str) -> &'static MenuStrings {
    LOCALES
        .iter()
        .find(|(name, _, _)| *name == locale)
        .map_or(&EN, |(_, _, strings)| strings)
}

/// Build a fresh menu: muda 0.17 on macOS does not clear the native key
/// equivalent when set_accelerator(None) is called on an existing item.
pub fn create_menu(app: &AppHandle<Wry>, bindings: &BTreeMap<String, String>) -> tauri::Result<()> {
    let l = menu_strings(menu_locale());
    let command = |id: &str, label: &str| {
        let mut item = MenuItemBuilder::new(label).id(id);
        if let Some(binding) = bindings.get(id).filter(|value| !value.is_empty()) {
            item = item.accelerator(binding.replace("Mod+", "CmdOrCtrl+"));
        }
        item.build(app)
    };
    let import_submenu = SubmenuBuilder::new(app, l.import)
        .item(&command("import_plottr", "Plottr (.pltr)")?)
        .item(&command("import_ywriter", "yWriter 7 (.yw7)")?)
        .item(&command("import_markdown", "Markdown (.md)")?)
        .item(&command("import_longform", l.import_longform)?)
        .item(&command("import_scrivener", "Scrivener 3 (.scriv)")?)
        .item(&command("import_novelwriter", l.import_novelwriter)?)
        .build()?;
    let file_submenu = SubmenuBuilder::new(app, l.file)
        .item(&command("new_project", l.new_project)?)
        .item(&command("editorial_open", l.editorial_open)?)
        .item(&command("editorial_project", l.editorial_project)?)
        .separator()
        .item(&import_submenu)
        .item(&command("export", l.export)?)
        .separator()
        .item(&command("close_project", l.close_project)?)
        .separator()
        .item(&command("settings", l.settings)?)
        .separator()
        .item(&command("quit", l.quit)?)
        .build()?;
    let edit_submenu = SubmenuBuilder::new(app, l.edit)
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .separator()
        .item(&command("find", l.find)?)
        .item(&command("find_replace", l.find_replace)?)
        .item(&command("find_project", l.find_project)?)
        .build()?;
    // Close Project owns Mod+W; a second predefined Close Window item would
    // retain that accelerator after the user's Close Project binding changes.
    //
    // muda does not implement the predefined Minimize and Maximize items on
    // Linux, so the menu rendered empty there. Linux gets ordinary items that
    // setup_menu_events handles natively.
    let window_submenu = if cfg!(target_os = "linux") {
        SubmenuBuilder::new(app, l.window)
            .item(
                &MenuItemBuilder::new(l.minimize)
                    .id(WINDOW_MINIMIZE)
                    .build(app)?,
            )
            .item(
                &MenuItemBuilder::new(l.maximize)
                    .id(WINDOW_TOGGLE_MAXIMIZE)
                    .build(app)?,
            )
            .build()?
    } else {
        SubmenuBuilder::new(app, l.window)
            .minimize()
            .maximize()
            .build()?
    };
    let view_submenu = SubmenuBuilder::new(app, l.view)
        .item(&command("toggle_sidebar", l.toggle_sidebar)?)
        .item(&command("toggle_references", l.toggle_references)?)
        .item(&command("sync", l.sync)?)
        .build()?;
    let help_submenu = SubmenuBuilder::new(app, l.help)
        .item(&command("about", l.about)?)
        .item(&command("send_feedback", l.send_feedback)?)
        .separator()
        .item(&command("command_palette", l.command_palette)?)
        .item(&command("quick_start", l.quick_start)?)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    impl MenuStrings {
        /// Every label, destructured exhaustively so a new field has to be listed here too.
        fn labels(&self) -> Vec<&'static str> {
            let MenuStrings {
                file,
                new_project,
                editorial_open,
                editorial_project,
                import,
                import_longform,
                import_novelwriter,
                export,
                close_project,
                settings,
                quit,
                edit,
                find,
                find_replace,
                find_project,
                view,
                toggle_sidebar,
                toggle_references,
                sync,
                window,
                minimize,
                maximize,
                help,
                about,
                send_feedback,
                command_palette,
                quick_start,
            } = self;
            vec![
                file,
                new_project,
                editorial_open,
                editorial_project,
                import,
                import_longform,
                import_novelwriter,
                export,
                close_project,
                settings,
                quit,
                edit,
                find,
                find_replace,
                find_project,
                view,
                toggle_sidebar,
                toggle_references,
                sync,
                window,
                minimize,
                maximize,
                help,
                about,
                send_feedback,
                command_palette,
                quick_start,
            ]
        }
    }

    #[test]
    fn every_label_is_filled_in_every_locale() {
        for (locale, _, strings) in LOCALES {
            for (index, label) in strings.labels().iter().enumerate() {
                assert!(!label.trim().is_empty(), "{locale} label {index} is empty");
            }
        }
    }

    #[test]
    fn english_labels_are_unchanged() {
        assert_eq!(
            EN.labels(),
            [
                "File",
                "New Project",
                "Open Review or Feedback File…",
                "Editorial Review…",
                "Import",
                "Longform (Index or Vault...)",
                "novelWriter (Project Folder)",
                "Export...",
                "Close Project",
                "Settings...",
                "Quit kindling",
                "Edit",
                "Find in Scene…",
                "Find and Replace…",
                "Find and Replace in Project…",
                "View",
                "Toggle Sidebar",
                "Toggle References Panel",
                "Sync from Source",
                "Window",
                "Minimize",
                "Maximize",
                "Help",
                "About kindling…",
                "Send Feedback...",
                "Command Palette...",
                "Quick Start",
            ]
        );
    }

    #[test]
    fn detection_matches_the_frontend_table() {
        let cases: BTreeMap<String, String> =
            serde_json::from_str(include_str!("../../src/lib/i18n/detection.json")).unwrap();
        assert!(!cases.is_empty());
        for (tag, locale) in &cases {
            assert_eq!(resolve_locale(tag), locale, "tag {tag:?}");
        }
    }

    #[test]
    fn every_frontend_catalog_has_menu_strings() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../src/lib/i18n/locales");
        let catalogs: BTreeSet<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap())
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        let menus: BTreeSet<String> = LOCALES
            .iter()
            .map(|(locale, _, _)| locale.to_string())
            .collect();
        assert_eq!(catalogs, menus);
    }

    #[test]
    fn kindling_locale_overrides_the_system_language() {
        let some = |tag: &str| Some(tag.to_string());
        assert_eq!(pick_locale(some("zh-CN"), some("en-US")), "zh-CN");
        assert_eq!(pick_locale(some("en"), some("zh-Hans-CN")), "en");
        assert_eq!(pick_locale(some(" "), some("zh-TW")), "zh-CN");
        assert_eq!(pick_locale(None, some("zh-TW")), "zh-CN");
        assert_eq!(pick_locale(None, None), "en");
    }

    #[test]
    fn menu_strings_follow_the_locale() {
        assert_eq!(menu_strings("zh-CN").file, "文件");
        assert_eq!(menu_strings("en").file, "File");
        assert_eq!(menu_strings("xx").file, "File");
    }
}
