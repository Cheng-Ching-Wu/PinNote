use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Wry};

use crate::i18n::pick;
use crate::services::{lifecycle, notes, updater, window};
use crate::tr;

const MENU_NEW_NOTE: &str = "new_note";
const MENU_OPEN_MAIN: &str = "open_main";
const MENU_SHOW_ALL: &str = "show_all";
const MENU_FROM_CLIPBOARD: &str = "from_clipboard";
const MENU_TOGGLE_HIDDEN: &str = "toggle_hidden";
const MENU_SEARCH: &str = "search";
const MENU_QUIT: &str = "quit";
const MENU_UPDATE: &str = "update";

const TRAY_ID: &str = "pinnote-tray";

/// Built in the current language; rebuilt by `refresh` when it (or the available update) changes.
fn build_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let item = |id: &str, text: &str| MenuItem::with_id(app, id, text, true, None::<&str>);
    let header = MenuItem::with_id(app, "header", "PinNote", false, None::<&str>)?;
    let new_note = item(MENU_NEW_NOTE, pick("新增便籤", "New note"))?;
    let open_main = item(MENU_OPEN_MAIN, pick("開啟便籤管理", "Open note manager"))?;
    let from_clipboard = item(
        MENU_FROM_CLIPBOARD,
        pick("從剪貼簿新增", "New note from clipboard"),
    )?;
    let search = item(MENU_SEARCH, pick("快速搜尋", "Quick search"))?;
    let show_all = item(MENU_SHOW_ALL, pick("顯示全部便籤", "Show all notes"))?;
    let toggle_hidden = item(
        MENU_TOGGLE_HIDDEN,
        pick("隱藏 / 顯示便籤", "Hide / show notes"),
    )?;
    let quit = item(MENU_QUIT, pick("結束", "Quit"))?;
    let menu = Menu::with_items(
        app,
        &[
            &header,
            &PredefinedMenuItem::separator(app)?,
            &new_note,
            &from_clipboard,
            &open_main,
            &search,
            &PredefinedMenuItem::separator(app)?,
            &show_all,
            &toggle_hidden,
            &PredefinedMenuItem::separator(app)?,
        ],
    )?;
    if let Some(update) = updater::available(app) {
        let text = tr!("更新到 v{}…", "Update to v{}…", update.version);
        menu.append(&item(MENU_UPDATE, &text)?)?;
    }
    menu.append(&quit)?;
    Ok(menu)
}

/// Applies the current language and update state to the tray menu.
pub fn refresh(app: &AppHandle) -> tauri::Result<()> {
    match app.tray_by_id(TRAY_ID) {
        Some(tray) => tray.set_menu(Some(build_menu(app)?)),
        None => Ok(()),
    }
}

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let menu = build_menu(app)?;
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
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
            MENU_SEARCH => {
                if let Err(err) = window::toggle_search_window(app) {
                    eprintln!("[pinnote] tray: failed to open search: {err}");
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
            MENU_UPDATE => {
                // The manager window shows the update dialog.
                if let Err(err) = window::show_main_window(app) {
                    eprintln!("[pinnote] tray: failed to show main window: {err}");
                }
                let _ = app.emit_to(window::MAIN_LABEL, updater::UPDATE_OPEN, ());
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
