import { invoke } from '@tauri-apps/api/core'

/** App-level settings that live in the OS / file system rather than in SQLite. */
export const settingsService = {
  getAutostart(): Promise<boolean> {
    return invoke('get_autostart')
  },

  /** Returns the state actually applied by the OS. */
  setAutostart(enabled: boolean): Promise<boolean> {
    return invoke('set_autostart', { enabled })
  },

  /** Opens an http(s) / mailto link in the default browser. */
  openExternal(url: string): Promise<void> {
    return invoke('open_external', { url })
  },

  /** Daily database snapshots (last 7 days) in `%APPDATA%\com.pinnote.app\backups`. */
  openBackupFolder(): Promise<void> {
    return invoke('open_backup_folder')
  },
}
