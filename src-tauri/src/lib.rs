mod commands;
mod database;
mod error;
mod events;
mod models;
mod services;
mod state;

use tauri::RunEvent;

use services::{lifecycle, window};
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        // Must be the first plugin: a second launch just focuses the running instance,
        // so the same note can never get windows from two processes.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = window::show_main_window(app);
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![lifecycle::AUTOSTART_ARG]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .setup(|app| {
            lifecycle::startup(app.handle())?;
            Ok(())
        })
        .on_window_event(lifecycle::handle_window_event)
        .invoke_handler(tauri::generate_handler![
            commands::notes::get_notes,
            commands::notes::get_note,
            commands::notes::create_note,
            commands::notes::update_note,
            commands::notes::archive_note,
            commands::notes::restore_note,
            commands::notes::soft_delete_note,
            commands::notes::restore_deleted_note,
            commands::notes::delete_note_permanently,
            commands::windows::open_note_window,
            commands::windows::close_note_window,
            commands::windows::show_all_notes,
            commands::windows::show_main_window,
            commands::app::get_shortcut_status,
            commands::app::get_autostart,
            commands::app::set_autostart,
            commands::app::export_notes,
            commands::app::import_notes,
            commands::app::open_backup_folder,
            commands::app::open_external,
        ])
        .build(tauri::generate_context!())
        .expect("error while building PinNote");

    app.run(|_app, event| {
        // Closing every window (main only hides) must not end the app —
        // it keeps running in the tray until "結束" is chosen.
        if let RunEvent::ExitRequested {
            code: None, api, ..
        } = event
        {
            api.prevent_exit();
        }
    });
}
