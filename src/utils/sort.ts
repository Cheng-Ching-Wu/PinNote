import type { Note } from '@/models/note'
import { noteHeading } from '@/utils/noteText'

export type NoteSort = 'updated' | 'created' | 'title' | 'manual'

export const NOTE_SORTS: readonly NoteSort[] = ['updated', 'created', 'title', 'manual']

export const NOTE_SORT_LABELS: Record<NoteSort, string> = {
  updated: '最後修改',
  created: '建立時間',
  title: '標題',
  manual: '手動排序',
}

export function isNoteSort(value: unknown): value is NoteSort {
  return typeof value === 'string' && (NOTE_SORTS as readonly string[]).includes(value)
}

const collator = new Intl.Collator('zh-Hant', { numeric: true, sensitivity: 'base' })

/** Newest first for dates; ties fall back to the last modified time so the order is stable. */
export function sortNotes(notes: readonly Note[], sort: NoteSort): Note[] {
  const byUpdated = (a: Note, b: Note) => b.updatedAt.localeCompare(a.updatedAt)
  const compare: Record<NoteSort, (a: Note, b: Note) => number> = {
    updated: byUpdated,
    created: (a, b) => b.createdAt.localeCompare(a.createdAt) || byUpdated(a, b),
    title: (a, b) => collator.compare(noteHeading(a), noteHeading(b)) || byUpdated(a, b),
    manual: (a, b) => a.sortOrder - b.sortOrder || byUpdated(a, b),
  }
  return [...notes].sort(compare[sort])
}

/** Moves `id` to sit right before `beforeId` (or to the end when null). */
export function moveId(ids: readonly string[], id: string, beforeId: string | null): string[] {
  const rest = ids.filter((other) => other !== id)
  const index = beforeId === null ? rest.length : rest.indexOf(beforeId)
  if (index === -1) return [...ids]
  return [...rest.slice(0, index), id, ...rest.slice(index)]
}

/**
 * Applies a new order of the visible notes (e.g. while a filter hides some) to
 * the whole list: hidden notes keep their slots, visible ones fill theirs in the new order.
 */
export function mergeVisibleOrder(allIds: readonly string[], visibleIds: readonly string[]): string[] {
  const visible = new Set(visibleIds)
  let next = 0
  return allIds.map((id) => (visible.has(id) ? visibleIds[next++] : id))
}
