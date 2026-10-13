import { describe, expect, it } from 'vitest'

import { autoTagColor, resolveTagColor } from '@/constants/tagColors'
import {
  collectTags,
  MAX_TAG_LENGTH,
  matchesTagFilter,
  normalizeTag,
  normalizeTags,
  parseTagQuery,
  UNTAGGED_KEY,
} from '@/utils/tags'

describe('normalizeTag', () => {
  it('cleans names like the Rust side', () => {
    expect(normalizeTag('  #工作  ')).toBe('工作')
    expect(normalizeTag('a   b\tc')).toBe('a b c')
    expect(normalizeTag('##')).toBeNull()
    expect(normalizeTag('   ')).toBeNull()
    expect(Array.from(normalizeTag('字'.repeat(40))!)).toHaveLength(MAX_TAG_LENGTH)
  })

  it('dedupes without case, keeping the first spelling and order', () => {
    expect(normalizeTags(['Work', '', '待辦', 'work', '#待辦', 'Home'])).toEqual(['Work', '待辦', 'Home'])
  })
})

describe('collectTags', () => {
  it('counts notes per tag and merges spellings', () => {
    const tags = collectTags([{ tags: ['work', 'home'] }, { tags: ['Work'] }, { tags: [] }])
    expect(tags).toEqual([
      { name: 'home', key: 'home', count: 1 },
      { name: 'work', key: 'work', count: 2 },
    ])
  })
})

describe('matchesTagFilter', () => {
  const tagged = { tags: ['Work', 'home'] }
  const untagged = { tags: [] }

  it('matches everything without a selection', () => {
    expect(matchesTagFilter(tagged, new Set())).toBe(true)
    expect(matchesTagFilter(untagged, new Set())).toBe(true)
  })

  it('uses OR between selected tags', () => {
    expect(matchesTagFilter(tagged, new Set(['work', 'other']))).toBe(true)
    expect(matchesTagFilter(tagged, new Set(['other']))).toBe(false)
  })

  it('selects untagged notes with UNTAGGED_KEY', () => {
    const selected = new Set([UNTAGGED_KEY, 'other'])
    expect(matchesTagFilter(untagged, selected)).toBe(true)
    expect(matchesTagFilter(tagged, selected)).toBe(false)
  })
})

describe('parseTagQuery', () => {
  it('splits #tags from the text', () => {
    expect(parseTagQuery('  #工作 會議  #Home 紀錄 ')).toEqual({ tags: ['工作', 'Home'], text: '會議 紀錄' })
    expect(parseTagQuery('#')).toEqual({ tags: [], text: '#' })
  })
})

describe('tag colors', () => {
  it('uses the chosen color, otherwise a stable automatic one', () => {
    expect(autoTagColor('Work')).toBe(autoTagColor('work'))
    expect(autoTagColor('work')).not.toBe('gray')
    expect(resolveTagColor('Work', { work: 'teal' })).toBe('teal')
    expect(resolveTagColor('work', { work: 'nope' })).toBe(autoTagColor('work'))
  })
})
