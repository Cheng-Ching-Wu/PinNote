import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { t } from '@/i18n'
import { type UpdateInfo, type UpdateProgress, updateService } from '@/services/updateService'
import { useSettingsStore } from '@/stores/settings'

export const useUpdateStore = defineStore('update', () => {
  const settings = useSettingsStore()

  const available = ref<UpdateInfo | null>(null)
  const currentVersion = ref('')
  const checking = ref(false)
  const installing = ref(false)
  const progress = ref<UpdateProgress | null>(null)
  const error = ref<string | null>(null)
  /** Result of a manual check that found nothing, e.g. "already the latest version". */
  const upToDate = ref(false)
  const dialogOpen = ref(false)

  /** The banner stays away for a version the user chose "later" for; a newer one shows again. */
  const showBanner = computed(
    () => !!available.value && available.value.version !== settings.settings.dismissedUpdate,
  )

  /** 0–100, or null while the size is unknown. */
  const percent = computed(() => {
    const p = progress.value
    return p?.total ? Math.min(100, Math.round((p.downloaded / p.total) * 100)) : null
  })

  async function load() {
    const [version, found] = await Promise.all([
      updateService.currentVersion().catch(() => ''),
      updateService.getAvailable().catch(() => null),
    ])
    currentVersion.value = version
    available.value = found
  }

  /** Manual check (settings menu): always opens the dialog to show the result. */
  async function check() {
    dialogOpen.value = true
    checking.value = true
    error.value = null
    upToDate.value = false
    try {
      available.value = await updateService.check()
      upToDate.value = !available.value
    } catch (err) {
      error.value = t('update.checkFailed', { error: String(err) })
    } finally {
      checking.value = false
    }
  }

  async function install() {
    if (!available.value || installing.value) return
    installing.value = true
    error.value = null
    progress.value = null
    try {
      // On success PinNote closes and the installer starts the new version.
      await updateService.install()
    } catch (err) {
      error.value = t('update.installFailed', { error: String(err) })
      installing.value = false
    }
  }

  function dismiss() {
    if (available.value) void settings.updateSettings({ dismissedUpdate: available.value.version })
  }

  function openDialog() {
    error.value = null
    upToDate.value = false
    dialogOpen.value = true
  }

  let unsubscribe: (() => void)[] = []

  /** Follows background checks and download progress. */
  async function startSync() {
    if (unsubscribe.length) return
    unsubscribe = await Promise.all([
      updateService.onAvailable((info) => (available.value = info)),
      updateService.onProgress((next) => (progress.value = next)),
      updateService.onOpenRequested(openDialog),
    ])
  }

  return {
    available,
    currentVersion,
    checking,
    installing,
    progress,
    percent,
    error,
    upToDate,
    dialogOpen,
    showBanner,
    load,
    check,
    install,
    dismiss,
    openDialog,
    startSync,
  }
})
