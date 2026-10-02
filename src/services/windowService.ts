import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
import type { UnlistenFn } from '@tauri-apps/api/event'

import { COLLAPSED_HEIGHT, NOTE_MIN_HEIGHT, NOTE_MIN_WIDTH } from '@/constants/window'
import type { WindowGeometry } from '@/models/note'
import { debounce } from '@/utils/debounce'

const NOTE_LABEL_PREFIX = 'note-'

export const windowService = {
  /** Opens the note window, or focuses it if it already exists (deduplicated in Rust). */
  openNoteWindow(id: string): Promise<void> {
    return invoke('open_note_window', { id })
  },

  closeNoteWindow(id: string): Promise<void> {
    return invoke('close_note_window', { id })
  },

  showMainWindow(): Promise<void> {
    return invoke('show_main_window')
  },

  /** Note id of the current webview, or null in the main window. */
  currentNoteId(): string | null {
    const label = getCurrentWindow().label
    return label.startsWith(NOTE_LABEL_PREFIX) ? label.slice(NOTE_LABEL_PREFIX.length) : null
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
