import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

import type { AppSettings } from '@/models/settings'

/** Event emitted by `src-tauri/src/events.rs`. */
const SETTINGS_CHANGED = 'settings:changed'

export const settingsRepository = {
  getSettings(): Promise<AppSettings> {
    return invoke('get_settings')
  },

  updateSettings(data: Partial<AppSettings>): Promise<AppSettings> {
    return invoke('update_settings', { data })
  },

  /** Every window (including note windows) receives setting changes. */
  subscribe(onChanged: (settings: AppSettings) => void): Promise<UnlistenFn> {
    return listen<AppSettings>(SETTINGS_CHANGED, (event) => onChanged(event.payload))
  },
}
