import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { type MessageKey, t } from '@/i18n'
import { shortcutService, type ShortcutStatus } from '@/services/shortcutService'

export const useShortcutStore = defineStore('shortcuts', () => {
  const statuses = ref<ShortcutStatus[]>([])

  /** Configured but not registered (e.g. owned by another app). */
  const failed = computed(() => statuses.value.filter((s) => s.accelerator && !s.registered))

  /** Translated here rather than in Rust, so it follows a language change right away. */
  function labelOf(status: ShortcutStatus): string {
    const key = `shortcuts.actions.${status.action}` as MessageKey
    const label = t(key)
    return label === key ? status.label : label
  }

  async function load() {
    statuses.value = await shortcutService.getStatus().catch(() => [])
  }

  /** Returns an error message, or null on success. */
  async function set(action: string, accelerator: string): Promise<string | null> {
    try {
      statuses.value = await shortcutService.setShortcut(action, accelerator)
      return null
    } catch (error) {
      return String(error)
    }
  }

  let paused = false

  async function pause() {
    if (paused) return
    paused = true
    await shortcutService.setPaused(true).catch(console.error)
  }

  async function resume() {
    if (!paused) return
    paused = false
    statuses.value = await shortcutService.setPaused(false).catch(() => statuses.value)
  }

  return { statuses, failed, labelOf, load, set, pause, resume }
})
