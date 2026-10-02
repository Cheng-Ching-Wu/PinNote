//! Creation / lookup of the main window and the per-note windows.

use tauri::{
    window::Color, AppHandle, LogicalSize, Manager, PhysicalPosition, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};

use crate::error::AppResult;
use crate::models::{Note, NoteColor, NoteUpdate};

pub const MAIN_LABEL: &str = "main";
const NOTE_LABEL_PREFIX: &str = "note-";

/// Height of a collapsed note: just the header bar (logical px).
/// Must match the header height in `NoteView.vue`.
pub const COLLAPSED_HEIGHT: f64 = 32.0;
pub const MIN_WIDTH: f64 = 180.0;
pub const MIN_HEIGHT: f64 = 96.0;

pub fn note_label(id: &str) -> String {
    format!("{NOTE_LABEL_PREFIX}{id}")
}

pub fn note_id_from_label(label: &str) -> Option<&str> {
    label.strip_prefix(NOTE_LABEL_PREFIX)
}

pub fn show_main_window(app: &AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window(MAIN_LABEL) {
        window.unminimize()?;
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }

    WebviewWindowBuilder::new(app, MAIN_LABEL, WebviewUrl::App("index.html".into()))
        .title("PinNote")
        .inner_size(380.0, 600.0)
        .min_inner_size(320.0, 400.0)
        .background_color(Color(250, 250, 249, 255))
        .center()
        .build()?;
    Ok(())
}

/// Opens (or focuses) the window of a note, restoring its saved geometry and state.
/// Idempotent: a note never gets more than one window.
pub fn open_note_window(app: &AppHandle, note: &Note, focus: bool) -> AppResult<WebviewWindow> {
    let label = note_label(&note.id);

    if let Some(window) = app.get_webview_window(&label) {
        window.unminimize()?;
        window.show()?;
        if focus {
            window.set_focus()?;
        }
        return Ok(window);
    }

    let width = note.width as f64;
    let height = if note.collapsed {
        COLLAPSED_HEIGHT
    } else {
        note.height as f64
    };
    let min_height = if note.collapsed {
        COLLAPSED_HEIGHT
    } else {
        MIN_HEIGHT
    };

    let saved_position = match (note.x, note.y) {
        (Some(x), Some(y)) => monitor_scale_at(app, x, y).map(|scale| (x, y, scale)),
        _ => None,
    };

    let mut builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
        .title(window_title(note))
        .inner_size(width, height)
        .min_inner_size(MIN_WIDTH, min_height)
        .decorations(false)
        .shadow(true)
        .resizable(!note.collapsed)
        .maximizable(false)
        .minimizable(false)
        .skip_taskbar(true)
        .always_on_top(note.always_on_top)
        .background_color(note_background(note.color))
        .focused(focus);
    builder = match saved_position {
        // Approximate placement up front so the window doesn't flash elsewhere first.
        Some((x, y, scale)) => builder.position(x as f64 / scale, y as f64 / scale),
        None => builder.center(),
    };
    let window = builder.build()?;

    // Exact placement must happen after the window is shown: while an undecorated
    // window is still hidden, Windows reports / applies its size including the
    // caption height, which would make the note 30px off.
    window.show()?;
    if let Some((x, y, _)) = saved_position {
        window.set_position(PhysicalPosition::new(x as i32, y as i32))?;
    }
    // Applied once the window sits on its target monitor, so mixed-DPI setups
    // also get the right physical size.
    window.set_size(LogicalSize::new(width, height))?;

    if focus {
        window.set_focus()?;
    }
    Ok(window)
}

pub fn close_note_window(app: &AppHandle, id: &str) -> AppResult<bool> {
    match app.get_webview_window(&note_label(id)) {
        Some(window) => {
            window.close()?;
            Ok(true)
        }
        None => Ok(false),
    }
}

pub fn sync_title(app: &AppHandle, note: &Note) {
    if let Some(window) = app.get_webview_window(&note_label(&note.id)) {
        let _ = window.set_title(&window_title(note));
    }
}

/// Reads the current geometry of a note window. Height is skipped while
/// collapsed so the expanded height is never overwritten.
pub fn capture_geometry(window: &WebviewWindow, collapsed: bool) -> AppResult<Option<NoteUpdate>> {
    if window.is_minimized()? {
        return Ok(None);
    }
    let position = window.outer_position()?;
    let scale = window.scale_factor()?;
    let size = window.inner_size()?.to_logical::<f64>(scale);

    Ok(Some(NoteUpdate {
        x: Some(position.x as i64),
        y: Some(position.y as i64),
        width: Some(size.width.round() as i64),
        height: (!collapsed).then(|| size.height.round() as i64),
        ..Default::default()
    }))
}

fn window_title(note: &Note) -> String {
    let title = note.title.trim();
    if title.is_empty() {
        "PinNote".into()
    } else {
        format!("{title} - PinNote")
    }
}

/// Initial webview background, so a note doesn't flash white while loading.
/// Keep in sync with `--note-body` in `src/styles/note-colors.css`.
fn note_background(color: NoteColor) -> Color {
    match color {
        NoteColor::Yellow => Color(255, 247, 191, 255),
        NoteColor::Blue => Color(220, 236, 252, 255),
        NoteColor::Green => Color(222, 245, 216, 255),
        NoteColor::Pink => Color(252, 225, 236, 255),
        NoteColor::Gray => Color(238, 238, 236, 255),
    }
}

/// Scale factor of the monitor showing the top-left area of a saved position,
/// or `None` when that point is on no connected monitor (e.g. the note was on a
/// now-disconnected display) — the window is then centred instead.
fn monitor_scale_at(app: &AppHandle, x: i64, y: i64) -> Option<f64> {
    let monitors = app.available_monitors().ok()?;
    let (probe_x, probe_y) = (x + 40, y + 10);
    monitors
        .iter()
        .find(|monitor| {
            let pos = monitor.position();
            let size = monitor.size();
            probe_x >= pos.x as i64
                && probe_y >= pos.y as i64
                && probe_x < pos.x as i64 + size.width as i64
                && probe_y < pos.y as i64 + size.height as i64
        })
        .map(|monitor| monitor.scale_factor())
}
