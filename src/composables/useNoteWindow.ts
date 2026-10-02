import { computed, onBeforeUnmount, onMounted } from 'vue'

import { GEOMETRY_SAVE_DELAY } from '@/constants/window'
import type { NoteColor, NoteUpdate, WindowGeometry } from '@/models/note'
import { windowService } from '@/services/windowService'
import { useNoteStore } from '@/stores/note'

/**
 * Window-level state of a note: geometry persistence, always-on-top,
 * collapse and color. Runs inside the note's own window.
 */
export function useNoteWindow(id: string) {
  const store = useNoteStore()
  const note = computed(() => store.getById(id))

  function onGeometry(geometry: WindowGeometry) {
    const current = note.value
    if (!current) return

    const patch: NoteUpdate = {}
    if (geometry.x !== current.x) patch.x = geometry.x
    if (geometry.y !== current.y) patch.y = geometry.y
    if (geometry.width !== current.width) patch.width = geometry.width
    // While collapsed the window is only a header: keep the expanded height.
    if (!current.collapsed && geometry.height !== current.height) patch.height = geometry.height

    if (Object.keys(patch).length > 0) void store.updateNote(id, patch)
  }

  let stopWatching: (() => void) | undefined
  let unmounted = false

  onMounted(async () => {
    const stop = await windowService.watchGeometry(onGeometry, GEOMETRY_SAVE_DELAY)
    if (unmounted) stop()
    else stopWatching = stop
  })

  onBeforeUnmount(() => {
    unmounted = true
    stopWatching?.()
  })

  async function toggleAlwaysOnTop() {
    const current = note.value
    if (!current) return
    const value = !current.alwaysOnTop
    await windowService.setAlwaysOnTop(value)
    await store.updateNote(id, { alwaysOnTop: value })
  }

  async function toggleCollapsed() {
    const current = note.value
    if (!current) return
    const collapsed = !current.collapsed
    // Update the store first (synchronously, optimistic) so the resize event
    // caused below already sees the new state and doesn't save the wrong height.
    const saving = store.updateNote(id, { collapsed })
    await windowService.applyCollapsed(collapsed, current.height)
    await saving
  }

  async function setColor(color: NoteColor) {
    await store.updateNote(id, { color })
  }

  return { note, toggleAlwaysOnTop, toggleCollapsed, setColor }
}
