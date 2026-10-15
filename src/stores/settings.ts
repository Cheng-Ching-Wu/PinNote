import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

import { type MessageKey, resolveLocale, setLocale, t } from '@/i18n'
import { type AppSettings, DEFAULT_SETTINGS } from '@/models/settings'
import { settingsRepository } from '@/repositories/settingsRepository'
import { settingsService } from '@/services/settingsService'
import { tagKey } from '@/utils/tags'

export const useSettingsStore = defineStore('settings', () => {
  /** Preferences stored in SQLite, shared by all windows. */
  const settings = ref<AppSettings>({ ...DEFAULT_SETTINGS })
  /** OS login item — read from the system, not stored in SQLite. */
  const autostart = ref(false)
  const error = ref<string | null>(null)

  // Every window follows the language setting, including changes made in another window.
  watch(
    () => settings.value.language,
    (language) => setLocale(resolveLocale(language)),
    { immediate: true },
  )

  async function guard(label: MessageKey, task: () => Promise<void>) {
    try {
      await task()
    } catch (err) {
      error.value = `${t(label)}: ${String(err)}`
      console.error(`[settings-store] ${label}`, err)
    }
  }

  const loadSettings = () =>
    guard('errors.loadSettings', async () => {
      settings.value = await settingsRepository.getSettings()
    })

  const loadAutostart = () =>
    guard('errors.loadAutostart', async () => {
      autostart.value = await settingsService.getAutostart()
    })

  const updateSettings = (data: Partial<AppSettings>) =>
    guard('errors.saveSettings', async () => {
      settings.value = { ...settings.value, ...data }
      settings.value = await settingsRepository.updateSettings(data)
    })

  /** `null` goes back to the automatic color. */
  function setTagColor(name: string, color: string | null) {
    const tagColors = { ...settings.value.tagColors }
    if (color) tagColors[tagKey(name)] = color
    else delete tagColors[tagKey(name)]
    return updateSettings({ tagColors })
  }

  const setAutostart = (enabled: boolean) =>
    guard('errors.setAutostart', async () => {
      autostart.value = await settingsService.setAutostart(enabled)
    })

  const openBackupFolder = () => guard('errors.openBackups', () => settingsService.openBackupFolder())

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
    setTagColor,
    setAutostart,
    openBackupFolder,
    startSync,
  }
})
