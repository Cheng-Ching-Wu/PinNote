//! Creation / lookup of the main window and the per-note windows.

use tauri::{
    window::Color, AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};

use crate::error::AppResult;
use crate::models::{Note, NoteUpdate};
use crate::state::AppState;

pub const MAIN_LABEL: &str = "main";
pub const SEARCH_LABEL: &str = "search";
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
        // HTML5 drag and drop (reordering the list) instead of native file drops.
        .disable_drag_drop_handler()
        .center()
        .build()?;
    Ok(())
}

/// Quick search (Ctrl+Alt+F): shows the search window, or hides it when it is
/// already in front. The window is kept alive (hidden) so it opens instantly.
pub fn toggle_search_window(app: &AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window(SEARCH_LABEL) {
        if window.is_visible()? && window.is_focused()? {
            window.hide()?;
            return Ok(());
        }
        window.center()?;
        window.show()?;
        window.set_focus()?;
        // The page resets its query and focuses the input.
        window.emit(SEARCH_SHOWN, ())?;
        return Ok(());
    }

    WebviewWindowBuilder::new(app, SEARCH_LABEL, WebviewUrl::App("index.html".into()))
        .title("PinNote 搜尋")
        .inner_size(520.0, 380.0)
        .decorations(false)
        .shadow(true)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .background_color(Color(255, 255, 255, 255))
        .center()
        .focused(true)
        .build()?;
    Ok(())
}

/// Emitted to the search window each time it is shown again.
pub const SEARCH_SHOWN: &str = "search:shown";

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
        // Transparent so pinned notes can fade (CSS opacity) to show what's
        // behind them; this also means no white flash while the page loads.
        .transparent(true)
        // Let the webview receive files dropped from Explorer (images into the note)
        // instead of Tauri's native drag-drop events.
        .disable_drag_drop_handler()
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

/// Ctrl+Alt+H: hides (or shows again) every note window without closing it,
/// so `is_open` and the desktop layout are untouched.
pub fn set_notes_hidden(app: &AppHandle, hidden: bool) -> AppResult<()> {
    for (label, window) in app.webview_windows() {
        if note_id_from_label(&label).is_none() {
            continue;
        }
        if hidden {
            window.hide()?;
        } else {
            window.show()?;
        }
    }
    app.state::<AppState>().set_notes_hidden(hidden);
    Ok(())
}

/// Returns the new state: `true` when the notes are now hidden.
pub fn toggle_notes_hidden(app: &AppHandle) -> AppResult<bool> {
    let hidden = !app.state::<AppState>().notes_hidden();
    set_notes_hidden(app, hidden)?;
    Ok(hidden)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArrangeLayout {
    /// Side by side in rows, keeping each note's size.
    Grid,
    /// Diagonally overlapping, each title bar still visible.
    Cascade,
}

/// Where on the desktop the arranged notes go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArrangeAnchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

/// Lines up every visible note window on the primary monitor's work area,
/// starting from `anchor`. The notes save their new positions themselves
/// (geometry watcher in each window). Returns how many windows were moved.
pub fn arrange_note_windows(
    app: &AppHandle,
    layout: ArrangeLayout,
    anchor: ArrangeAnchor,
) -> AppResult<usize> {
    let Some(monitor) = app.primary_monitor()? else {
        return Ok(0);
    };
    let scale = monitor.scale_factor();
    let area = monitor.work_area();
    let (left, top) = (area.position.x as i64, area.position.y as i64);
    let (right, bottom) = (left + area.size.width as i64, top + area.size.height as i64);

    // Current reading order (top to bottom, left to right) so notes move as little as possible.
    let mut windows = Vec::new();
    for (label, window) in app.webview_windows() {
        if note_id_from_label(&label).is_some() && window.is_visible()? {
            let position = window.outer_position()?;
            let size = window.outer_size()?;
            windows.push((window, position, size));
        }
    }
    windows.sort_by_key(|(_, position, _)| (position.y, position.x));

    let px = |logical: f64| (logical * scale).round() as i64;
    let (margin, gap, step) = (px(16.0), px(12.0), px(32.0));
    let area = (left + margin, top + margin, right - margin, bottom - margin);
    let sizes: Vec<(i64, i64)> = windows
        .iter()
        .map(|(_, _, size)| (size.width as i64, size.height as i64))
        .collect();
    let positions = match layout {
        ArrangeLayout::Grid => {
            let positions = grid_positions(sizes.iter().copied(), area, gap, step);
            anchor_positions(positions, &sizes, area, anchor)
        }
        ArrangeLayout::Cascade => {
            let tallest = sizes.iter().map(|(_, h)| *h).max().unwrap_or(0);
            let widest = sizes.iter().map(|(w, _)| *w).max().unwrap_or(0);
            let positions = cascade_positions(sizes.len(), area, step, tallest);
            // Mirror by a common box so the title bars stay evenly stepped.
            anchor_positions(
                positions,
                &vec![(widest, tallest); sizes.len()],
                area,
                anchor,
            )
        }
    };
    for ((window, _, _), (x, y)) in windows.iter().zip(positions) {
        window.set_position(PhysicalPosition::new(x as i32, y as i32))?;
    }
    Ok(windows.len())
}

/// Rows from the top-left; when the area is full, starts over from the top, shifted by `step`.
fn grid_positions(
    sizes: impl Iterator<Item = (i64, i64)>,
    (left, top, right, bottom): (i64, i64, i64, i64),
    gap: i64,
    step: i64,
) -> Vec<(i64, i64)> {
    let mut positions = Vec::new();
    let (mut origin_x, mut origin_y) = (left, top);
    let (mut x, mut y, mut row_height) = (origin_x, origin_y, 0);
    for (width, height) in sizes {
        if x > origin_x && x + width > right {
            x = origin_x;
            y += row_height + gap;
            row_height = 0;
        }
        if y > origin_y && y + height > bottom {
            origin_x += step;
            origin_y += step;
            x = origin_x;
            y = origin_y;
            row_height = 0;
        }
        positions.push((x, y));
        x += width + gap;
        row_height = row_height.max(height);
    }
    positions
}

/// Moves a layout computed from the top-left corner to `anchor`: right / bottom
/// corners mirror it (so it grows inwards from that corner), `Center` centres its bounds.
fn anchor_positions(
    positions: Vec<(i64, i64)>,
    sizes: &[(i64, i64)],
    (left, top, right, bottom): (i64, i64, i64, i64),
    anchor: ArrangeAnchor,
) -> Vec<(i64, i64)> {
    let mirror_x = matches!(anchor, ArrangeAnchor::TopRight | ArrangeAnchor::BottomRight);
    let mirror_y = matches!(
        anchor,
        ArrangeAnchor::BottomLeft | ArrangeAnchor::BottomRight
    );

    if anchor == ArrangeAnchor::Center {
        let rects = positions.iter().zip(sizes);
        let min_x = rects.clone().map(|((x, _), _)| *x).min().unwrap_or(left);
        let min_y = rects.clone().map(|((_, y), _)| *y).min().unwrap_or(top);
        let max_x = rects
            .clone()
            .map(|((x, _), (w, _))| x + w)
            .max()
            .unwrap_or(right);
        let max_y = rects.map(|((_, y), (_, h))| y + h).max().unwrap_or(bottom);
        // Never push the top-left out of the area when the layout is larger than it.
        let dx = (left + (right - left - (max_x - min_x)) / 2 - min_x).max(left - min_x);
        let dy = (top + (bottom - top - (max_y - min_y)) / 2 - min_y).max(top - min_y);
        return positions
            .into_iter()
            .map(|(x, y)| (x + dx, y + dy))
            .collect();
    }

    positions
        .into_iter()
        .zip(sizes)
        .map(|((x, y), (w, h))| {
            let x = if mirror_x { right - (x - left) - w } else { x };
            let y = if mirror_y { bottom - (y - top) - h } else { y };
            (x.max(left), y.max(top))
        })
        .collect()
}

/// Diagonal steps; a new column starts when the next note would leave the area.
fn cascade_positions(
    count: usize,
    (left, top, right, bottom): (i64, i64, i64, i64),
    step: i64,
    tallest: i64,
) -> Vec<(i64, i64)> {
    let per_column = (((bottom - top - tallest) / step.max(1)) + 1).max(1) as usize;
    (0..count)
        .map(|i| {
            let column = (i / per_column) as i64;
            let row = (i % per_column) as i64;
            let x = (left + column * step * 4 + row * step).min(right - step);
            (x, top + row * step)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_wraps_rows_and_restarts_when_full() {
        let area = (0, 0, 250, 250);
        let positions = grid_positions([(100, 100); 6].into_iter(), area, 10, 20);
        assert_eq!(
            positions,
            vec![(0, 0), (110, 0), (0, 110), (110, 110), (20, 20), (130, 20)]
        );
    }

    #[test]
    fn anchors_mirror_or_centre_the_layout() {
        let area = (0, 0, 1000, 800);
        let positions = vec![(0, 0), (110, 0)];
        let sizes = [(100, 50), (100, 80)];
        let at = |anchor| anchor_positions(positions.clone(), &sizes, area, anchor);

        assert_eq!(at(ArrangeAnchor::TopLeft), vec![(0, 0), (110, 0)]);
        assert_eq!(at(ArrangeAnchor::TopRight), vec![(900, 0), (790, 0)]);
        assert_eq!(at(ArrangeAnchor::BottomLeft), vec![(0, 750), (110, 720)]);
        assert_eq!(at(ArrangeAnchor::BottomRight), vec![(900, 750), (790, 720)]);
        // Bounds 210 x 80 centred in 1000 x 800.
        assert_eq!(at(ArrangeAnchor::Center), vec![(395, 360), (505, 360)]);
    }

    #[test]
    fn cascade_steps_diagonally_and_starts_new_columns() {
        let positions = cascade_positions(4, (0, 0, 1000, 160), 30, 100);
        assert_eq!(positions, vec![(0, 0), (30, 30), (60, 60), (120, 0)]);
    }
}
