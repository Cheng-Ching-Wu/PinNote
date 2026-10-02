// Keep in sync with `src-tauri/src/services/window.rs`.

/** Height of a collapsed note window (= header bar), logical px. */
export const COLLAPSED_HEIGHT = 32
export const NOTE_MIN_WIDTH = 180
export const NOTE_MIN_HEIGHT = 96

/** Debounce for persisting window move / resize. */
export const GEOMETRY_SAVE_DELAY = 300
/** Debounce for persisting typed title / content. */
export const CONTENT_SAVE_DELAY = 400
