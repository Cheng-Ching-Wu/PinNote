import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

/** Events emitted by `src-tauri/src/services/updater.rs`. */
const UPDATE_AVAILABLE = 'update:available'
const UPDATE_PROGRESS = 'update:progress'
const UPDATE_OPEN = 'update:open'
const NOTES_FLUSH = 'notes:flush'

export interface UpdateInfo {
  version: string
  currentVersion: string
  /** Release notes. */
  notes: string
  date: string | null
}

export interface UpdateProgress {
  downloaded: number
  /** null when the size isn't known. */
  total: number | null
}

/**
 * In-app updates from GitHub Releases. Checks run in Rust (at startup and daily,
 * even with no window open); the manager window shows and installs them.
 */
export const updateService = {
  currentVersion(): Promise<string> {
    return getVersion()
  },

  /** What the last check found, without asking the server again. */
  getAvailable(): Promise<UpdateInfo | null> {
    return invoke('get_available_update')
  },

  /** Asks the release server; null = already the latest version. */
  check(): Promise<UpdateInfo | null> {
    return invoke('check_for_update')
  },

  /** Downloads and installs; PinNote closes and the new version starts. */
  install(): Promise<void> {
    return invoke('install_update')
  },

  onAvailable(handler: (info: UpdateInfo) => void): Promise<UnlistenFn> {
    return listen<UpdateInfo>(UPDATE_AVAILABLE, (event) => handler(event.payload))
  },

  onProgress(handler: (progress: UpdateProgress) => void): Promise<UnlistenFn> {
    return listen<UpdateProgress>(UPDATE_PROGRESS, (event) => handler(event.payload))
  },

  /** Tray → "update to vX". */
  onOpenRequested(handler: () => void): Promise<UnlistenFn> {
    return listen(UPDATE_OPEN, () => handler())
  },

  /** Right before installing: note windows save pending typing. */
  onFlushRequested(handler: () => void): Promise<UnlistenFn> {
    return listen(NOTES_FLUSH, () => handler())
  },
}
