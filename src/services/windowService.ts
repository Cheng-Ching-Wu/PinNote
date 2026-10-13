import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
import type { UnlistenFn } from '@tauri-apps/api/event'

import type { ArrangeAnchor, ArrangeLayout } from '@/constants/arrange'
import { COLLAPSED_HEIGHT, NOTE_MIN_HEIGHT, NOTE_MIN_WIDTH } from '@/constants/window'
import type { WindowGeometry } from '@/models/note'
import { debounce } from '@/utils/debounce'

const NOTE_LABEL_PREFIX = 'note-'
const SEARCH_LABEL = 'search'
/** Emitted by `toggle_search_window` in `src-tauri/src/services/window.rs`. */
const SEARCH_SHOWN = 'search:shown'

export const windowService = {
  /** Opens the note window, or focuses it if it already exists (deduplicated in Rust). */
  openNoteWindow(id: string): Promise<void> {
    return invoke('open_note_window', { id })
  },

  closeNoteWindow(id: string): Promise<void> {
    return invoke('close_note_window', { id })
  },

  /** Reopens every active note on the desktop; resolves to the number shown. */
  showAllNotes(): Promise<number> {
    return invoke('show_all_notes')
  },

  /** Lines up the visible note windows on the primary monitor from `anchor`; resolves to how many moved. */
  arrangeNoteWindows(layout: ArrangeLayout, anchor: ArrangeAnchor): Promise<number> {
    return invoke('arrange_note_windows', { layout, anchor })
  },

  /** Opens exactly these notes on the desktop and closes the rest; resolves to how many are shown. */
  showOnlyNotes(ids: string[]): Promise<number> {
    return invoke('show_only_notes', { ids })
  },

  /** Hides / shows every note window (Ctrl+Alt+H); resolves to true when now hidden. */
  toggleNotesHidden(): Promise<boolean> {
    return invoke('toggle_notes_hidden')
  },

  showMainWindow(): Promise<void> {
    return invoke('show_main_window')
  },

  /** Shows the quick search window, or hides it when it is already in front. */
  toggleSearchWindow(): Promise<void> {
    return invoke('toggle_search_window')
  },

  /** True in the quick search window (label `search`, created in Rust). */
  isSearchWindow(): boolean {
    return getCurrentWindow().label === SEARCH_LABEL
  },

  /** Fired each time the (kept-alive) search window is shown again. */
  onSearchShown(handler: () => void): Promise<UnlistenFn> {
    return getCurrentWindow().listen(SEARCH_SHOWN, () => handler())
  },

  hideCurrentWindow(): Promise<void> {
    return getCurrentWindow().hide()
  },

  /** Note id of the current webview, or null in the main window. */
  currentNoteId(): string | null {
    const label = getCurrentWindow().label
    return label.startsWith(NOTE_LABEL_PREFIX) ? label.slice(NOTE_LABEL_PREFIX.length) : null
  },

  onFocusChanged(handler: (focused: boolean) => void): Promise<UnlistenFn> {
    return getCurrentWindow().onFocusChanged(({ payload }) => handler(payload))
  },

  setAlwaysOnTop(value: boolean): Promise<void> {
    return getCurrentWindow().setAlwaysOnTop(value)
  },

  /**
   * Shrinks the current window to its header, or restores `expandedHeight`.
   * Width is kept as-is.
   */
  async applyCollapsed(collapsed: boolean, expandedHeight: number): Promise<void> {
    const win = getCurrentWindow()
    const width = (await win.innerSize()).toLogical(await win.scaleFactor()).width

    if (collapsed) {
      await win.setMinSize(new LogicalSize(NOTE_MIN_WIDTH, COLLAPSED_HEIGHT))
      await win.setSize(new LogicalSize(width, COLLAPSED_HEIGHT))
      await win.setResizable(false)
    } else {
      await win.setResizable(true)
      await win.setSize(new LogicalSize(width, Math.max(expandedHeight, NOTE_MIN_HEIGHT)))
      await win.setMinSize(new LogicalSize(NOTE_MIN_WIDTH, NOTE_MIN_HEIGHT))
    }
  },

  /**
   * Reports the current window geometry after it stops moving / resizing for `delay` ms.
   * Position is physical px (multi-monitor safe), size is logical px.
   */
  async watchGeometry(onChange: (geometry: WindowGeometry) => void, delay: number): Promise<UnlistenFn> {
    const win = getCurrentWindow()

    const report = debounce(async () => {
      if (await win.isMinimized()) return
      const [position, size, scale] = await Promise.all([win.outerPosition(), win.innerSize(), win.scaleFactor()])
      const logical = size.toLogical(scale)
      onChange({
        x: position.x,
        y: position.y,
        width: Math.round(logical.width),
        height: Math.round(logical.height),
      })
    }, delay)

    const unlistenMoved = await win.onMoved(() => report())
    const unlistenResized = await win.onResized(() => report())

    return () => {
      report.cancel()
      unlistenMoved()
      unlistenResized()
    }
  },
}
