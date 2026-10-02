import { computed, onBeforeUnmount, onMounted, ref, type Ref } from 'vue'

import type { Note } from '@/models/note'
import { windowService } from '@/services/windowService'
import { useSettingsStore } from '@/stores/settings'

/**
 * A pinned note fades while it is neither focused nor hovered, so it stays
 * visible for reference without hiding what's behind it.
 * Requires the note window to be transparent (see `open_note_window` in Rust).
 */
export function useNoteTranslucency(note: Ref<Note | undefined>) {
  const settings = useSettingsStore()

  const focused = ref(document.hasFocus())
  const hovered = ref(false)

  const faded = computed(
    () =>
      !!note.value?.alwaysOnTop &&
      settings.settings.pinnedTranslucent &&
      !focused.value &&
      !hovered.value,
  )

  let stopListening: (() => void) | undefined
  let unmounted = false

  onMounted(async () => {
    void settings.loadSettings()
    const stop = await windowService.onFocusChanged((value) => (focused.value = value))
    if (unmounted) stop()
    else stopListening = stop
  })

  onBeforeUnmount(() => {
    unmounted = true
    stopListening?.()
  })

  return {
    faded,
    onPointerEnter: () => (hovered.value = true),
    onPointerLeave: () => (hovered.value = false),
  }
}
