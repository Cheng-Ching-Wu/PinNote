import { describe, expect, it } from 'vitest'
import { ref } from 'vue'

import { useNoteSelection } from '@/composables/useNoteSelection'
import type { Note } from '@/models/note'
import { searchNotes } from '@/utils/search'
import { mergeVisibleOrder, moveId, sortNotes } from '@/utils/sort'

function makeNote(id: string, overrides: Partial<Note> = {}): Note {
  return {
    id,
    title: '',
    content: '',
    color: 'yellow',
    tags: [],
    x: null,
    y: null,
    width: 320,
    height: 280,
    alwaysOnTop: false,
    collapsed: false,
    isOpen: true,
    status: 'active',
    sortOrder: 0,
    createdAt: '2026-10-01T00:00:00.000Z',
    updatedAt: '2026-10-01T00:00:00.000Z',
    deletedAt: null,
    modifiedAt: '2026-10-01T00:00:00.000Z',
    ...overrides,
  }
}

const ids = (notes: { id: string }[]) => notes.map((n) => n.id)

describe('sortNotes', () => {
  const a = makeNote('a', { title: 'banana', createdAt: '2026-01-03', updatedAt: '2026-02-01', sortOrder: 2 })
  const b = makeNote('b', { title: 'apple', createdAt: '2026-01-01', updatedAt: '2026-02-03', sortOrder: 0 })
  const c = makeNote('c', { content: 'Cherry 2\nmore', createdAt: '2026-01-02', updatedAt: '2026-02-02', sortOrder: 1 })

  it('sorts by each mode without mutating the input', () => {
    const notes = [a, b, c]
    expect(ids(sortNotes(notes, 'updated'))).toEqual(['b', 'c', 'a'])
    expect(ids(sortNotes(notes, 'created'))).toEqual(['a', 'c', 'b'])
    expect(ids(sortNotes(notes, 'manual'))).toEqual(['b', 'c', 'a'])
    // Untitled notes sort by their first line ("Cherry 2").
    expect(ids(sortNotes(notes, 'title'))).toEqual(['b', 'a', 'c'])
    expect(ids(notes)).toEqual(['a', 'b', 'c'])
  })
})

describe('reordering', () => {
  it('moves an id before another, or to the end', () => {
    expect(moveId(['a', 'b', 'c', 'd'], 'd', 'b')).toEqual(['a', 'd', 'b', 'c'])
    expect(moveId(['a', 'b', 'c'], 'a', null)).toEqual(['b', 'c', 'a'])
    expect(moveId(['a', 'b', 'c'], 'a', 'a')).toEqual(['a', 'b', 'c'])
  })

  it('keeps hidden notes in their slots', () => {
    // b and d are filtered out; the visible a, c, e become e, a, c.
    expect(mergeVisibleOrder(['a', 'b', 'c', 'd', 'e'], ['e', 'a', 'c'])).toEqual(['e', 'b', 'a', 'd', 'c'])
  })
})

describe('searchNotes', () => {
  const notes = [
    makeNote('content-hit', { content: '標題\n明天的會議記錄', updatedAt: '2026-03-01' }),
    makeNote('title-hit', { title: '會議', updatedAt: '2026-01-01' }),
    makeNote('archived', { title: '會議 舊', status: 'archived', updatedAt: '2026-05-01' }),
    makeNote('trash', { title: '會議 刪除', status: 'deleted' }),
    makeNote('tagged', { title: 'x', tags: ['工作'], updatedAt: '2026-04-01' }),
  ]

  it('ranks heading hits first, active before archived, and skips the trash', () => {
    const results = searchNotes(notes, '會議')
    expect(ids(results.map((r) => r.note))).toEqual(['title-hit', 'archived', 'content-hit'])
    expect(results[2].snippet).toBe('明天的會議記錄')
  })

  it('supports #tags and lists recent active notes for an empty query', () => {
    expect(ids(searchNotes(notes, '#工作').map((r) => r.note))).toEqual(['tagged'])
    expect(ids(searchNotes(notes, '', 2).map((r) => r.note))).toEqual(['tagged', 'content-hit'])
  })
})

describe('useNoteSelection', () => {
  it('toggles, selects ranges and drops ids that leave the list', async () => {
    const list = ref(['a', 'b', 'c', 'd'])
    const selection = useNoteSelection(list)

    selection.toggle('b')
    selection.selectRange('d')
    expect(selection.ids.value).toEqual(['b', 'c', 'd'])
    selection.toggle('c')
    expect(selection.ids.value).toEqual(['b', 'd'])

    list.value = ['a', 'b']
    await Promise.resolve()
    expect(selection.ids.value).toEqual(['b'])
    expect(selection.count.value).toBe(1)

    selection.selectAll()
    expect(selection.allSelected.value).toBe(true)
    selection.clear()
    expect(selection.selecting.value).toBe(false)
  })
})
