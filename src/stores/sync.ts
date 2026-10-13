import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

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

  async function guard(label: string, task: () => Promise<SyncStatus | null>) {
    error.value = null
    try {
      const next = await task()
      if (next) status.value = next
    } catch (err) {
      error.value = `${label}: ${String(err)}`
    }
  }

  const load = () => guard('讀取同步狀態失敗', () => syncService.getStatus())
  const chooseFolder = () => guard('設定同步資料夾失敗', () => syncService.chooseFolder())
  const disable = () => guard('停止同步失敗', () => syncService.disable())
  const syncNow = () => guard('同步失敗', () => syncService.syncNow())
  const recreateFolder = () => guard('重新建立同步資料夾失敗', () => syncService.recreateFolder())

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
