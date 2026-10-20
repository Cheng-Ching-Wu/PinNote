//! Snapping for note windows: while a note is dragged or resized, an edge that
//! comes close to another note stops `LAYOUT_GAP` away from it, and an edge
//! near the screen stops `LAYOUT_MARGIN` from the work area — the same spacing
//! "arrange" uses. Hold Alt to place a note freely.
//!
//! The math (`snap`) is plain and tested; on Windows it runs inside the native
//! move / size loop (`WM_MOVING` / `WM_SIZING`), so notes snap while dragging.

use std::sync::atomic::{AtomicBool, Ordering};

/// How close (logical px) an edge must get to a spot before it snaps there.
pub const SNAP_THRESHOLD: f64 = 8.0;
/// How far (logical px) an edge may run past a spot and still be held there.
/// Kept small, so a note lets go as soon as it is pulled onward.
pub const SNAP_OVERSHOOT: f64 = 3.0;

static ENABLED: AtomicBool = AtomicBool::new(true);

/// Follows the `snap_windows` setting.
pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Screen rectangle in physical px; `right` / `bottom` are exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// Which edges follow the pointer: all of them when moving, the dragged ones when resizing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edges {
    pub left: bool,
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
}

impl Edges {
    pub const MOVE: Self = Self {
        left: true,
        top: true,
        right: true,
        bottom: true,
    };
}

/// Distances in physical px.
#[derive(Debug, Clone, Copy)]
pub struct Spacing {
    /// Reach toward a spot ahead of the moving edge.
    pub threshold: i32,
    /// Reach toward a spot the edge has already passed.
    pub overshoot: i32,
    /// Between two notes.
    pub gap: i32,
    /// Between a note and the work-area edge.
    pub margin: i32,
}

/// One axis of a rectangle.
#[derive(Debug, Clone, Copy)]
struct Span {
    lo: i32,
    hi: i32,
}

fn x_span(rect: &Rect) -> Span {
    Span {
        lo: rect.left,
        hi: rect.right,
    }
}

fn y_span(rect: &Rect) -> Span {
    Span {
        lo: rect.top,
        hi: rect.bottom,
    }
}

/// Where the low and high edge of `me` may go on one axis; `cross` is the other axis.
fn axis_targets(
    me_cross: Span,
    others: impl Iterator<Item = (Span, Span)>,
    work: Span,
    spacing: Spacing,
) -> (Vec<i32>, Vec<i32>) {
    let mut lo = vec![work.lo + spacing.margin];
    let mut hi = vec![work.hi - spacing.margin];
    for (main, cross) in others {
        // Side by side on this axis: keep the gap between facing edges.
        if me_cross.lo < cross.hi && me_cross.hi > cross.lo {
            lo.push(main.hi + spacing.gap);
            hi.push(main.lo - spacing.gap);
        }
        // Stacked on the other axis, about a gap apart: line up the same edges.
        let apart = (me_cross.lo - cross.hi).max(cross.lo - me_cross.hi);
        if (apart - spacing.gap).abs() <= spacing.threshold {
            lo.push(main.lo);
            hi.push(main.hi);
        }
    }
    (lo, hi)
}

/// Direction the pointer moves on each axis: -1, 0 (not moving on it) or 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Motion {
    pub x: i32,
    pub y: i32,
}

/// Offset that moves `value` onto the closest target in reach. A spot ahead
/// of the motion pulls from `threshold` away; one already passed only holds
/// for `overshoot`, so the edge doesn't get stuck on it.
fn nearest(value: i32, targets: &[i32], direction: i32, spacing: Spacing) -> Option<i32> {
    targets
        .iter()
        .map(|target| target - value)
        .filter(|&delta| {
            let passed = direction != 0 && delta.signum() == -direction;
            delta.abs()
                <= if passed {
                    spacing.overshoot
                } else {
                    spacing.threshold
                }
        })
        .min_by_key(|delta| delta.abs())
}

/// Moves the low / high edges of one axis. When both edges move it is a drag
/// and the size is kept; otherwise only the dragged edge moves, never below `min`.
fn snap_axis(
    span: Span,
    move_lo: bool,
    move_hi: bool,
    (lo_targets, hi_targets): (Vec<i32>, Vec<i32>),
    direction: i32,
    spacing: Spacing,
    min: i32,
) -> Span {
    let lo = move_lo
        .then(|| nearest(span.lo, &lo_targets, direction, spacing))
        .flatten();
    let hi = move_hi
        .then(|| nearest(span.hi, &hi_targets, direction, spacing))
        .flatten();
    if move_lo && move_hi {
        let delta = match (lo, hi) {
            (Some(a), Some(b)) => {
                if a.abs() <= b.abs() {
                    a
                } else {
                    b
                }
            }
            (Some(d), None) | (None, Some(d)) => d,
            (None, None) => 0,
        };
        return Span {
            lo: span.lo + delta,
            hi: span.hi + delta,
        };
    }
    let mut out = span;
    if let Some(delta) = lo.filter(|delta| span.hi - (span.lo + delta) >= min) {
        out.lo += delta;
    }
    if let Some(delta) = hi.filter(|delta| span.hi + delta - span.lo >= min) {
        out.hi += delta;
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Left,
    Top,
    Right,
    Bottom,
}

impl Edge {
    fn of(self, rect: &Rect) -> i32 {
        match self {
            Edge::Left => rect.left,
            Edge::Top => rect.top,
            Edge::Right => rect.right,
            Edge::Bottom => rect.bottom,
        }
    }
}

/// An edge that was already snapped when the drag started. It ignores that
/// spot until it has moved more than the threshold away, so a snapped note can
/// be pulled loose right away instead of fighting the snap first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Release {
    pub edge: Edge,
    pub at: i32,
}

/// Snap targets of `rect`: left, right, top, bottom.
fn edge_targets(
    rect: &Rect,
    others: &[Rect],
    work: Rect,
    spacing: Spacing,
) -> [(Edge, Vec<i32>); 4] {
    let (left, right) = axis_targets(
        y_span(rect),
        others.iter().map(|o| (x_span(o), y_span(o))),
        x_span(&work),
        spacing,
    );
    let (top, bottom) = axis_targets(
        x_span(rect),
        others.iter().map(|o| (y_span(o), x_span(o))),
        y_span(&work),
        spacing,
    );
    [
        (Edge::Left, left),
        (Edge::Right, right),
        (Edge::Top, top),
        (Edge::Bottom, bottom),
    ]
}

/// The edges of `rect` that sit exactly on a snap target (taken when a drag starts).
pub fn snapped_edges(rect: Rect, others: &[Rect], work: Rect, spacing: Spacing) -> Vec<Release> {
    edge_targets(&rect, others, work, spacing)
        .into_iter()
        .flat_map(|(edge, targets)| {
            let at = edge.of(&rect);
            targets.contains(&at).then_some(Release { edge, at })
        })
        .collect()
}

/// Drops the releases whose edge has left its spot, so that spot snaps again.
pub fn update_releases(released: &mut Vec<Release>, rect: &Rect, threshold: i32) {
    released.retain(|release| (release.edge.of(rect) - release.at).abs() <= threshold);
}

/// Snaps `rect` (the note's visible bounds) to the other notes and the work area,
/// skipping the `released` spots. `motion` is where the pointer is heading.
#[allow(clippy::too_many_arguments)]
pub fn snap(
    rect: Rect,
    edges: Edges,
    motion: Motion,
    others: &[Rect],
    work: Rect,
    spacing: Spacing,
    min_size: (i32, i32),
    released: &[Release],
) -> Rect {
    let [left, right, top, bottom] =
        edge_targets(&rect, others, work, spacing).map(|(edge, mut targets)| {
            targets.retain(|&at| !released.contains(&Release { edge, at }));
            targets
        });
    let x = snap_axis(
        x_span(&rect),
        edges.left,
        edges.right,
        (left, right),
        motion.x,
        spacing,
        min_size.0,
    );
    let y = snap_axis(
        y_span(&rect),
        edges.top,
        edges.bottom,
        (top, bottom),
        motion.y,
        spacing,
        min_size.1,
    );
    Rect {
        left: x.lo,
        top: y.lo,
        right: x.hi,
        bottom: y.hi,
    }
}

#[cfg(windows)]
pub use win::attach;

/// Snapping hooks into the Win32 move / size loop; other platforms move freely.
#[cfg(not(windows))]
pub fn attach(_window: &tauri::WebviewWindow) {}

#[cfg(windows)]
mod win {
    use std::ffi::c_void;
    use std::sync::Mutex;

    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
    use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromRect, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows::Win32::UI::HiDpi::GetDpiForWindow;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_MENU};
    use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowRect, IsIconic, IsWindowVisible, WM_ENTERSIZEMOVE, WM_EXITSIZEMOVE, WM_MOVING,
        WM_NCDESTROY, WM_SIZING,
    };

    use super::{
        enabled, snap, snapped_edges, update_releases, Edges, Motion, Rect, Release, Spacing,
        SNAP_OVERSHOOT, SNAP_THRESHOLD,
    };
    // Kept in sync with "arrange", so snapped and arranged notes look the same.
    use crate::services::window::{LAYOUT_GAP, LAYOUT_MARGIN, MIN_HEIGHT, MIN_WIDTH};

    const SUBCLASS_ID: usize = 0x5049_4e53; // "PINS"

    // `wParam` of WM_SIZING: the edge or corner being dragged.
    const WMSZ_LEFT: usize = 1;
    const WMSZ_RIGHT: usize = 2;
    const WMSZ_TOP: usize = 3;
    const WMSZ_TOPLEFT: usize = 4;
    const WMSZ_TOPRIGHT: usize = 5;
    const WMSZ_BOTTOM: usize = 6;
    const WMSZ_BOTTOMLEFT: usize = 7;
    const WMSZ_BOTTOMRIGHT: usize = 8;

    /// Note windows (raw HWNDs, so the list can live in a static).
    static NOTES: Mutex<Vec<isize>> = Mutex::new(Vec::new());
    /// The note being moved / resized right now.
    static DRAG: Mutex<Option<Drag>> = Mutex::new(None);

    struct Drag {
        hwnd: isize,
        /// Visible bounds of the other notes, taken once when the drag starts.
        others: Vec<Rect>,
        /// Visible bounds minus window rect: the invisible resize border around the note.
        inset: Rect,
        /// Edges that started out snapped and haven't left their spot yet.
        released: Vec<Release>,
        /// Where the pointer put the note last time (before snapping), to tell its direction.
        last: Rect,
    }

    /// Hooks snapping into the note window's move / size loop.
    pub fn attach(window: &tauri::WebviewWindow) {
        let Ok(hwnd) = window.hwnd() else { return };
        let raw = hwnd.0 as isize;
        // Subclassing only works from the thread that owns the window.
        let _ = window.run_on_main_thread(move || install(raw));
    }

    fn install(raw: isize) {
        let Ok(mut notes) = NOTES.lock() else { return };
        if notes.contains(&raw) {
            return;
        }
        let ok = unsafe { SetWindowSubclass(to_hwnd(raw), Some(subclass_proc), SUBCLASS_ID, 0) };
        if ok.as_bool() {
            notes.push(raw);
        }
    }

    fn to_hwnd(raw: isize) -> HWND {
        HWND(raw as *mut c_void)
    }

    fn from_win(rect: RECT) -> Rect {
        Rect {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        }
    }

    fn window_rect(hwnd: HWND) -> Option<Rect> {
        let mut rect = RECT::default();
        unsafe { GetWindowRect(hwnd, &mut rect) }.ok()?;
        Some(from_win(rect))
    }

    /// What the user sees: without the invisible border Windows may add around the window.
    fn visible_rect(hwnd: HWND) -> Option<Rect> {
        let mut rect = RECT::default();
        let size = std::mem::size_of::<RECT>() as u32;
        let ok = unsafe {
            DwmGetWindowAttribute(
                hwnd,
                DWMWA_EXTENDED_FRAME_BOUNDS,
                &mut rect as *mut RECT as *mut c_void,
                size,
            )
        };
        match ok {
            Ok(()) => Some(from_win(rect)),
            Err(_) => window_rect(hwnd),
        }
    }

    fn begin_drag(hwnd: HWND) {
        let raw = hwnd.0 as isize;
        let (Some(window), Some(visible)) = (window_rect(hwnd), visible_rect(hwnd)) else {
            return;
        };
        let inset = Rect {
            left: visible.left - window.left,
            top: visible.top - window.top,
            right: visible.right - window.right,
            bottom: visible.bottom - window.bottom,
        };
        let others: Vec<Rect> = NOTES
            .lock()
            .map(|notes| {
                notes
                    .iter()
                    .filter(|&&other| other != raw)
                    .map(|&other| to_hwnd(other))
                    .filter(|&other| unsafe {
                        IsWindowVisible(other).as_bool() && !IsIconic(other).as_bool()
                    })
                    .filter_map(visible_rect)
                    .collect()
            })
            .unwrap_or_default();
        let released = match work_area(&visible) {
            Some(work) => snapped_edges(visible, &others, work, spacing(hwnd)),
            None => Vec::new(),
        };
        if let Ok(mut drag) = DRAG.lock() {
            *drag = Some(Drag {
                hwnd: raw,
                others,
                inset,
                released,
                last: visible,
            });
        }
    }

    fn scale(hwnd: HWND) -> f64 {
        match unsafe { GetDpiForWindow(hwnd) } {
            0 => 1.0,
            dpi => dpi as f64 / 96.0,
        }
    }

    fn spacing(hwnd: HWND) -> Spacing {
        let scale = scale(hwnd);
        let px = |logical: f64| (logical * scale).round() as i32;
        Spacing {
            threshold: px(SNAP_THRESHOLD),
            overshoot: px(SNAP_OVERSHOOT),
            gap: px(LAYOUT_GAP),
            margin: px(LAYOUT_MARGIN),
        }
    }

    fn end_drag() {
        if let Ok(mut drag) = DRAG.lock() {
            *drag = None;
        }
    }

    /// Work area (screen minus taskbar) of the monitor showing most of `rect`.
    fn work_area(rect: &Rect) -> Option<Rect> {
        let rect = RECT {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        };
        let monitor = unsafe { MonitorFromRect(&rect, MONITOR_DEFAULTTONEAREST) };
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        unsafe { GetMonitorInfoW(monitor, &mut info) }
            .as_bool()
            .then(|| from_win(info.rcWork))
    }

    fn alt_held() -> bool {
        let state = unsafe { GetKeyState(i32::from(VK_MENU.0)) };
        state < 0
    }

    /// Rewrites the rectangle Windows is about to apply (`lParam` of WM_MOVING / WM_SIZING).
    fn adjust(hwnd: HWND, lparam: LPARAM, edges: Edges) {
        if !enabled() || alt_held() || lparam.0 == 0 {
            return;
        }
        let Ok(mut drag) = DRAG.lock() else { return };
        let Some(drag) = drag.as_mut().filter(|drag| drag.hwnd == hwnd.0 as isize) else {
            return;
        };
        let proposed = unsafe { &mut *(lparam.0 as *mut RECT) };
        let inset = drag.inset;
        let visible = Rect {
            left: proposed.left + inset.left,
            top: proposed.top + inset.top,
            right: proposed.right + inset.right,
            bottom: proposed.bottom + inset.bottom,
        };
        let Some(work) = work_area(&visible) else {
            return;
        };

        let spacing = spacing(hwnd);
        let scale = scale(hwnd);
        let min_size = (
            (MIN_WIDTH * scale).round() as i32,
            (MIN_HEIGHT * scale).round() as i32,
        );
        // The dragged edge on each axis tells the direction (both edges move together when moving).
        let last = drag.last;
        let motion = Motion {
            x: if edges.left {
                visible.left - last.left
            } else {
                visible.right - last.right
            }
            .signum(),
            y: if edges.top {
                visible.top - last.top
            } else {
                visible.bottom - last.bottom
            }
            .signum(),
        };
        drag.last = visible;
        update_releases(&mut drag.released, &visible, spacing.threshold);
        let snapped = snap(
            visible,
            edges,
            motion,
            &drag.others,
            work,
            spacing,
            min_size,
            &drag.released,
        );
        proposed.left = snapped.left - inset.left;
        proposed.top = snapped.top - inset.top;
        proposed.right = snapped.right - inset.right;
        proposed.bottom = snapped.bottom - inset.bottom;
    }

    fn sizing_edges(wparam: WPARAM) -> Edges {
        let edge = wparam.0;
        Edges {
            left: matches!(edge, WMSZ_LEFT | WMSZ_TOPLEFT | WMSZ_BOTTOMLEFT),
            top: matches!(edge, WMSZ_TOP | WMSZ_TOPLEFT | WMSZ_TOPRIGHT),
            right: matches!(edge, WMSZ_RIGHT | WMSZ_TOPRIGHT | WMSZ_BOTTOMRIGHT),
            bottom: matches!(edge, WMSZ_BOTTOM | WMSZ_BOTTOMLEFT | WMSZ_BOTTOMRIGHT),
        }
    }

    unsafe extern "system" fn subclass_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _id: usize,
        _data: usize,
    ) -> LRESULT {
        match msg {
            WM_ENTERSIZEMOVE => begin_drag(hwnd),
            WM_EXITSIZEMOVE => end_drag(),
            WM_MOVING => adjust(hwnd, lparam, Edges::MOVE),
            WM_SIZING => adjust(hwnd, lparam, sizing_edges(wparam)),
            WM_NCDESTROY => {
                let raw = hwnd.0 as isize;
                let _ = RemoveWindowSubclass(hwnd, Some(subclass_proc), SUBCLASS_ID);
                if let Ok(mut notes) = NOTES.lock() {
                    notes.retain(|&note| note != raw);
                }
            }
            _ => {}
        }
        DefSubclassProc(hwnd, msg, wparam, lparam)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPACING: Spacing = Spacing {
        threshold: 8,
        overshoot: 3,
        gap: 12,
        margin: 16,
    };
    const WORK: Rect = Rect {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1040,
    };
    const MIN: (i32, i32) = (180, 96);

    fn rect(left: i32, top: i32, width: i32, height: i32) -> Rect {
        Rect {
            left,
            top,
            right: left + width,
            bottom: top + height,
        }
    }

    fn drag(moving: Rect, others: &[Rect]) -> Rect {
        let still = Motion::default();
        snap(moving, Edges::MOVE, still, others, WORK, SPACING, MIN, &[])
    }

    #[test]
    fn stops_a_gap_away_from_a_note_beside_it() {
        let other = rect(500, 300, 200, 200);
        // Right of `other` (ends at 700): 700 + 12 = 712.
        assert_eq!(drag(rect(718, 330, 200, 150), &[other]).left, 712);
        assert_eq!(drag(rect(705, 330, 200, 150), &[other]).left, 712);
        // Left of it: right edge at 500 - 12 = 488.
        assert_eq!(drag(rect(290, 330, 200, 150), &[other]).right, 488);
    }

    #[test]
    fn keeps_the_size_while_moving() {
        let other = rect(500, 300, 200, 200);
        let moved = drag(rect(718, 330, 200, 150), &[other]);
        assert_eq!(
            (moved.right - moved.left, moved.bottom - moved.top),
            (200, 150)
        );
    }

    #[test]
    fn a_snapped_note_stays_put() {
        let other = rect(500, 300, 200, 200);
        let snapped = drag(rect(718, 330, 200, 150), &[other]);
        assert_eq!(drag(snapped, &[other]), snapped);
    }

    #[test]
    fn ignores_notes_out_of_reach() {
        let other = rect(500, 300, 200, 200);
        let free = rect(740, 330, 200, 150);
        assert_eq!(drag(free, &[other]), free);
        // Level with it but far below: not side by side, so no gap snapping.
        let below = rect(715, 800, 200, 150);
        assert_eq!(drag(below, &[other]).left, 715);
    }

    #[test]
    fn lines_up_with_a_note_above_or_below() {
        let other = rect(500, 300, 200, 200);
        // Just below `other` (ends at 500), roughly a gap apart and nearly left-aligned.
        let moved = drag(rect(506, 515, 200, 150), &[other]);
        assert_eq!((moved.left, moved.top), (500, 512));
        // Right edges line up too.
        let moved = drag(rect(595, 515, 100, 150), &[other]);
        assert_eq!(moved.right, 700);
    }

    #[test]
    fn keeps_a_margin_from_the_screen_edges() {
        let moved = drag(rect(9, 1040 - 150 - 20, 200, 150), &[]);
        assert_eq!((moved.left, moved.bottom), (16, 1040 - 16));
    }

    #[test]
    fn resizing_only_moves_the_dragged_edge() {
        let other = rect(500, 300, 200, 200);
        let edges = Edges {
            left: false,
            top: false,
            right: true,
            bottom: false,
        };
        let resized = snap(
            rect(200, 330, 283, 150),
            edges,
            Motion { x: 1, y: 0 },
            &[other],
            WORK,
            SPACING,
            MIN,
            &[],
        );
        assert_eq!(resized, rect(200, 330, 288, 150));
    }

    #[test]
    fn resizing_never_goes_below_the_minimum_size() {
        let other = rect(500, 300, 200, 200);
        let edges = Edges {
            left: true,
            top: false,
            right: false,
            bottom: false,
        };
        // Snapping the left edge to 712 would leave 175px, under the 180px minimum.
        let proposed = rect(704, 330, 183, 150);
        let motion = Motion { x: -1, y: 0 };
        let resized = snap(proposed, edges, motion, &[other], WORK, SPACING, MIN, &[]);
        assert_eq!(resized, proposed);
    }

    /// Drags a note that starts at `start` through the proposed `lefts`, like the move loop does.
    fn drag_through(start: Rect, others: &[Rect], lefts: &[i32]) -> Vec<i32> {
        let mut released = snapped_edges(start, others, WORK, SPACING);
        let mut last = start.left;
        lefts
            .iter()
            .map(|&left| {
                let proposed = Rect {
                    left,
                    right: left + (start.right - start.left),
                    ..start
                };
                let motion = Motion {
                    x: (left - last).signum(),
                    y: 0,
                };
                last = left;
                update_releases(&mut released, &proposed, SPACING.threshold);
                snap(
                    proposed,
                    Edges::MOVE,
                    motion,
                    others,
                    WORK,
                    SPACING,
                    MIN,
                    &released,
                )
                .left
            })
            .collect()
    }

    #[test]
    fn pulls_ahead_but_lets_go_soon_after_passing() {
        let other = rect(500, 300, 200, 200);
        // Moving right toward 712: pulled in from 704, held until 3px past it.
        let lefts = drag_through(
            rect(690, 330, 200, 150),
            &[other],
            &[700, 706, 712, 714, 716],
        );
        assert_eq!(lefts, vec![700, 712, 712, 712, 716]);
    }

    #[test]
    fn pulling_away_from_a_spot_lets_go_soon() {
        let other = rect(500, 300, 200, 200);
        // Snapped at 712 on the way left, then dragged back right past the overshoot.
        let lefts = drag_through(rect(730, 330, 200, 150), &[other], &[716, 714, 716]);
        assert_eq!(lefts, vec![712, 712, 716]);
    }

    #[test]
    fn remembers_which_edges_start_out_snapped() {
        let other = rect(500, 300, 200, 200);
        let released = snapped_edges(rect(712, 330, 200, 150), &[other], WORK, SPACING);
        assert_eq!(
            released,
            vec![Release {
                edge: Edge::Left,
                at: 712
            }]
        );
    }

    #[test]
    fn a_snapped_note_pulls_loose_right_away() {
        let other = rect(500, 300, 200, 200);
        // Without the release, 713..722 would all be pulled back to 712.
        let lefts = drag_through(rect(712, 330, 200, 150), &[other], &[713, 716, 720]);
        assert_eq!(lefts, vec![713, 716, 720]);
    }

    #[test]
    fn snaps_again_after_leaving_and_coming_back() {
        let other = rect(500, 300, 200, 200);
        let lefts = drag_through(rect(712, 330, 200, 150), &[other], &[716, 740, 716]);
        assert_eq!(lefts, vec![716, 740, 712]);
    }
}
