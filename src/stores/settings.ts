import { defineStore } from 'pinia'
import { ref } from 'vue'

import { settingsService } from '@/services/settingsService'

export const useSettingsStore = defineStore('settings', () => {
  const autostart = ref(false)
  const error = ref<string | null>(null)

  async function load() {
    try {
      autostart.value = await settingsService.getAutostart()
    } catch (err) {
      error.value = `讀取開機啟動設定失敗: ${String(err)}`
    }
  }

  async function setAutostart(enabled: boolean) {
    try {
      autostart.value = await settingsService.setAutostart(enabled)
      error.value = null
    } catch (err) {
      error.value = `變更開機啟動失敗: ${String(err)}`
    }
  }

  async function openBackupFolder() {
    try {
      await settingsService.openBackupFolder()
    } catch (err) {
      error.value = `無法開啟備份資料夾: ${String(err)}`
    }
  }

  return { autostart, error, load, setAutostart, openBackupFolder }
})
