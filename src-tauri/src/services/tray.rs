use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::AppHandle;

use crate::services::{lifecycle, notes, window};

const MENU_NEW_NOTE: &str = "new_note";
const MENU_OPEN_MAIN: &str = "open_main";
const MENU_SHOW_ALL: &str = "show_all";
const MENU_FROM_CLIPBOARD: &str = "from_clipboard";
const MENU_TOGGLE_HIDDEN: &str = "toggle_hidden";
const MENU_QUIT: &str = "quit";

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let header = MenuItem::with_id(app, "header", "PinNote", false, None::<&str>)?;
    let new_note = MenuItem::with_id(app, MENU_NEW_NOTE, "新增便籤", true, None::<&str>)?;
    let open_main = MenuItem::with_id(app, MENU_OPEN_MAIN, "開啟便籤管理", true, None::<&str>)?;
    let from_clipboard =
        MenuItem::with_id(app, MENU_FROM_CLIPBOARD, "從剪貼簿新增", true, None::<&str>)?;
    let show_all = MenuItem::with_id(app, MENU_SHOW_ALL, "顯示全部便籤", true, None::<&str>)?;
    let toggle_hidden = MenuItem::with_id(
        app,
        MENU_TOGGLE_HIDDEN,
        "隱藏 / 顯示便籤",
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "結束", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &header,
            &PredefinedMenuItem::separator(app)?,
            &new_note,
            &from_clipboard,
            &open_main,
            &PredefinedMenuItem::separator(app)?,
            &show_all,
            &toggle_hidden,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    let mut builder = TrayIconBuilder::with_id("pinnote-tray")
        .tooltip("PinNote")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            MENU_NEW_NOTE => {
                if let Err(err) = notes::create_and_open(app) {
                    eprintln!("[pinnote] tray: failed to create note: {err}");
                }
            }
            MENU_OPEN_MAIN => {
                if let Err(err) = window::show_main_window(app) {
                    eprintln!("[pinnote] tray: failed to show main window: {err}");
                }
            }
            MENU_FROM_CLIPBOARD => {
                if let Err(err) = notes::create_from_clipboard(app) {
                    eprintln!("[pinnote] tray: failed to create note from clipboard: {err}");
                }
            }
            MENU_TOGGLE_HIDDEN => {
                if let Err(err) = window::toggle_notes_hidden(app) {
                    eprintln!("[pinnote] tray: failed to toggle notes: {err}");
                }
            }
            MENU_SHOW_ALL => {
                if let Err(err) = notes::show_all(app) {
                    eprintln!("[pinnote] tray: failed to show notes: {err}");
                }
            }
            MENU_QUIT => lifecycle::quit(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = window::show_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}
