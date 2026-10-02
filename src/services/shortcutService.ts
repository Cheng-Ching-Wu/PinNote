import { invoke } from '@tauri-apps/api/core'

export interface ShortcutStatus {
  accelerator: string
  registered: boolean
  error: string | null
}

/**
 * The global shortcut (Ctrl+Alt+N) is registered natively at startup so it
 * works even when no webview is alive; this service exposes its outcome.
 */
export const shortcutService = {
  getStatus(): Promise<ShortcutStatus | null> {
    return invoke('get_shortcut_status')
  },
}
