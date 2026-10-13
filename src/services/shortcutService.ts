import { invoke } from '@tauri-apps/api/core'

export interface ShortcutStatus {
  /** Stable id: 'new_note' | 'new_from_clipboard' | 'toggle_hidden' | 'quick_search'. */
  action: string
  label: string
  /** '' when disabled. */
  accelerator: string
  defaultAccelerator: string
  registered: boolean
  error: string | null
}

/**
 * Global shortcuts are registered natively (they work even when no webview is
 * alive). Each action's accelerator is user-configurable and saved in SQLite.
 */
export const shortcutService = {
  getStatus(): Promise<ShortcutStatus[]> {
    return invoke('get_shortcut_status')
  },

  /** Rejects (keeping the old one active) on conflicts or when another app owns the keys. */
  setShortcut(action: string, accelerator: string): Promise<ShortcutStatus[]> {
    return invoke('set_shortcut', { action, accelerator })
  },

  /** Pause while recording so pressing the current combination doesn't trigger it. */
  setPaused(paused: boolean): Promise<ShortcutStatus[]> {
    return invoke('set_shortcuts_paused', { paused })
  },
}
