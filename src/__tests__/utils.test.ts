import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import type { Note } from '@/models/note'
import { matchesQuery } from '@/stores/note'
import { debounce } from '@/utils/debounce'
import { formatNoteTime } from '@/utils/time'

function makeNote(overrides: Partial<Note> = {}): Note {
  return {
    id: 'n1',
    title: 'API 待確認',
    content: 'RTPMS\n碼頭 API\n航行時間 API',
    color: 'yellow',
    x: null,
    y: null,
    width: 320,
    height: 280,
    alwaysOnTop: false,
    collapsed: false,
    isOpen: true,
    status: 'active',
    createdAt: '2026-10-01T00:00:00.000Z',
    updatedAt: '2026-10-01T00:00:00.000Z',
    deletedAt: null,
    ...overrides,
  }
}

describe('matchesQuery', () => {
  it('matches title and content, case-insensitively', () => {
    const note = makeNote()
    expect(matchesQuery(note, '待確認')).toBe(true)
    expect(matchesQuery(note, 'rtpms')).toBe(true)
    expect(matchesQuery(note, '航行')).toBe(true)
    expect(matchesQuery(note, '甘特圖')).toBe(false)
  })

  it('treats a blank query as match-all', () => {
    expect(matchesQuery(makeNote(), '   ')).toBe(true)
  })
})

describe('formatNoteTime', () => {
  const now = new Date(2026, 9, 2, 12, 0)

  it('formats today / yesterday / older dates', () => {
    expect(formatNoteTime(new Date(2026, 9, 2, 8, 32).toISOString(), now)).toBe('今天 08:32')
    expect(formatNoteTime(new Date(2026, 9, 1, 17, 20).toISOString(), now)).toBe('昨天 17:20')
    expect(formatNoteTime(new Date(2026, 8, 30, 9, 5).toISOString(), now)).toBe('2026/09/30 09:05')
  })

  it('returns an empty string for invalid input', () => {
    expect(formatNoteTime('nope', now)).toBe('')
  })
})

describe('debounce', () => {
  beforeEach(() => vi.useFakeTimers())
  afterEach(() => vi.useRealTimers())

  it('only runs the last call after the wait', () => {
    const fn = vi.fn()
    const debounced = debounce(fn, 300)
    debounced(1)
    debounced(2)
    expect(debounced.pending).toBe(true)
    vi.advanceTimersByTime(299)
    expect(fn).not.toHaveBeenCalled()
    vi.advanceTimersByTime(1)
    expect(fn).toHaveBeenCalledOnce()
    expect(fn).toHaveBeenCalledWith(2)
    expect(debounced.pending).toBe(false)
  })

  it('can be cancelled', () => {
    const fn = vi.fn()
    const debounced = debounce(fn, 300)
    debounced()
    debounced.cancel()
    vi.advanceTimersByTime(1000)
    expect(fn).not.toHaveBeenCalled()
  })
})
