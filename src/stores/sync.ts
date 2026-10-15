import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { type MessageKey, t } from '@/i18n'
import { type SyncStatus, syncService } from '@/services/syncService'

export const useSyncStore = defineStore('sync', () => {
  const status = ref<SyncStatus>({
    folder: '',
    running: false,
    lastSyncAt: null,
    lastError: null,
    lastReport: null,
    folderMissing: false,
  })
  const error = ref<string | null>(null)

  const enabled = computed(() => !!status.value.folder)

  async function guard(label: MessageKey, task: () => Promise<SyncStatus | null>) {
    error.value = null
    try {
      const next = await task()
      if (next) status.value = next
    } catch (err) {
      error.value = `${t(label)}: ${String(err)}`
    }
  }

  const load = () => guard('errors.loadSync', () => syncService.getStatus())
  const chooseFolder = () => guard('errors.chooseSyncFolder', () => syncService.chooseFolder())
  const disable = () => guard('errors.disableSync', () => syncService.disable())
  const syncNow = () => guard('errors.sync', () => syncService.syncNow())
  const recreateFolder = () => guard('errors.recreateSyncFolder', () => syncService.recreateFolder())

  let unsubscribe: (() => void) | null = null

  /**
   * Follows the background runs. `onFinished` gets each completed run's status
   * (e.g. to report conflicts).
   */
  async function startSync(onFinished?: (status: SyncStatus) => void) {
    if (unsubscribe) return
    unsubscribe = await syncService.onStatus((next) => {
      const finished = status.value.running && !next.running
      status.value = next
      if (finished) onFinished?.(next)
    })
  }

  return { status, error, enabled, load, chooseFolder, disable, syncNow, recreateFolder, startSync }
})
