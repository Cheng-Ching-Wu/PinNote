import { defineStore } from 'pinia'
import { ref } from 'vue'

import { type AppSettings, DEFAULT_SETTINGS } from '@/models/settings'
import { settingsRepository } from '@/repositories/settingsRepository'
import { settingsService } from '@/services/settingsService'

export const useSettingsStore = defineStore('settings', () => {
  /** Preferences stored in SQLite, shared by all windows. */
  const settings = ref<AppSettings>({ ...DEFAULT_SETTINGS })
  /** OS login item — read from the system, not stored in SQLite. */
  const autostart = ref(false)
  const error = ref<string | null>(null)

  async function guard(label: string, task: () => Promise<void>) {
    try {
      await task()
    } catch (err) {
      error.value = `${label}: ${String(err)}`
      console.error(`[settings-store] ${label}`, err)
    }
  }

  const loadSettings = () =>
    guard('讀取設定失敗', async () => {
      settings.value = await settingsRepository.getSettings()
    })

  const loadAutostart = () =>
    guard('讀取開機啟動設定失敗', async () => {
      autostart.value = await settingsService.getAutostart()
    })

  const updateSettings = (data: Partial<AppSettings>) =>
    guard('儲存設定失敗', async () => {
      settings.value = { ...settings.value, ...data }
      settings.value = await settingsRepository.updateSettings(data)
    })

  const setAutostart = (enabled: boolean) =>
    guard('變更開機啟動失敗', async () => {
      autostart.value = await settingsService.setAutostart(enabled)
    })

  const openBackupFolder = () => guard('無法開啟備份資料夾', () => settingsService.openBackupFolder())

  let unsubscribe: (() => void) | null = null

  /** Keeps this window's copy in sync with changes made in any window. */
  async function startSync() {
    if (unsubscribe) return
    unsubscribe = await settingsRepository.subscribe((next) => (settings.value = next))
  }

  return {
    settings,
    autostart,
    error,
    loadSettings,
    loadAutostart,
    updateSettings,
    setAutostart,
    openBackupFolder,
    startSync,
  }
})
