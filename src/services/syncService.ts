import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

/** Events emitted by `src-tauri/src/events.rs`. */
const SYNC_STATUS = 'sync:status'
const NOTES_SYNCED = 'notes:synced'

export interface SyncReport {
  uploaded: number
  downloaded: number
  deleted: number
  /** Notes edited on two devices; the older text was kept as a revision. */
  conflicts: number
  images: number
  /** Problems with single files that didn't stop the run. */
  warnings: string[]
}

export interface SyncStatus {
  /** '' when sync is off. */
  folder: string
  running: boolean
  lastSyncAt: string | null
  lastError: string | null
  lastReport: SyncReport | null
  /** The folder synced before is gone; sync is paused until the user decides. */
  folderMissing: boolean
}

/**
 * Folder sync through a cloud drive (Google Drive, OneDrive…). Runs in Rust on a
 * background thread; progress arrives as `sync:status` events.
 */
export const syncService = {
  getStatus(): Promise<SyncStatus> {
    return invoke('get_sync_status')
  },

  /** Folder picker; resolves to null when cancelled. Turns sync on and starts a run. */
  chooseFolder(): Promise<SyncStatus | null> {
    return invoke('choose_sync_folder')
  },

  /** Turns sync off; the folder is left untouched. */
  disable(): Promise<SyncStatus> {
    return invoke('disable_sync')
  },

  /** After the folder went missing: recreate it and upload this device's notes. */
  recreateFolder(): Promise<SyncStatus> {
    return invoke('recreate_sync_folder')
  },

  syncNow(): Promise<SyncStatus> {
    return invoke('sync_now')
  },

  onStatus(handler: (status: SyncStatus) => void): Promise<UnlistenFn> {
    return listen<SyncStatus>(SYNC_STATUS, (event) => handler(event.payload))
  },

  /** Ids of notes whose text another device changed (open note windows reload). */
  onNotesSynced(handler: (ids: string[]) => void): Promise<UnlistenFn> {
    return listen<string[]>(NOTES_SYNCED, (event) => handler(event.payload))
  },
}

/** Cloud drive folders sync to other computers; anything else probably doesn't. */
export function looksLikeCloudFolder(path: string): boolean {
  return /google ?drive|my drive|我的雲端硬碟|共用雲端硬碟|shared drives|onedrive|dropbox|icloud/i.test(path)
}
