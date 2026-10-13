import { ref, type Ref } from 'vue'

import { moveId } from '@/utils/sort'

export type DropPosition = 'before' | 'after'

/**
 * HTML5 drag and drop to reorder a list. Bind the returned handlers on each item;
 * `onReorder` receives the ids in their new order.
 * (The main window disables Tauri's native file-drop handler so these events fire.)
 */
export function useListReorder(ids: Readonly<Ref<readonly string[]>>, onReorder: (ids: string[]) => void) {
  const dragging = ref<string | null>(null)
  const target = ref<{ id: string; position: DropPosition } | null>(null)

  function dropPositionOf(id: string): DropPosition | null {
    if (!dragging.value || target.value?.id !== id || id === dragging.value) return null
    return target.value.position
  }

  function onDragStart(event: DragEvent, id: string) {
    dragging.value = id
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move'
      event.dataTransfer.setData('text/plain', id)
    }
  }

  function onDragOver(event: DragEvent, id: string) {
    if (!dragging.value) return
    event.preventDefault()
    if (event.dataTransfer) event.dataTransfer.dropEffect = 'move'
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
    const position: DropPosition = event.clientY < rect.top + rect.height / 2 ? 'before' : 'after'
    if (target.value?.id !== id || target.value.position !== position) target.value = { id, position }
  }

  function onDrop(event: DragEvent) {
    event.preventDefault()
    const id = dragging.value
    const drop = target.value
    reset()
    if (!id || !drop || drop.id === id) return

    const list = ids.value
    const beforeId = drop.position === 'before' ? drop.id : (list[list.indexOf(drop.id) + 1] ?? null)
    const next = moveId(list, id, beforeId)
    if (next.some((value, index) => value !== list[index])) onReorder(next)
  }

  function reset() {
    dragging.value = null
    target.value = null
  }

  return { dragging, dropPositionOf, onDragStart, onDragOver, onDrop, onDragEnd: reset }
}
