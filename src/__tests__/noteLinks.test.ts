import { describe, expect, it } from 'vitest'

import { findLinkQuery, noteIdFromHref, noteLinkHref } from '@/utils/noteLinks'

describe('note links', () => {
  it('round-trips ids through the href', () => {
    const href = noteLinkHref('3f2a-uuid')
    expect(href).toBe('pinnote://note/3f2a-uuid')
    expect(noteIdFromHref(href)).toBe('3f2a-uuid')
    expect(noteIdFromHref('https://example.com')).toBeNull()
    expect(noteIdFromHref('pinnote://note/')).toBeNull()
    expect(noteIdFromHref(undefined)).toBeNull()
  })

  it('detects an unfinished [[ before the cursor', () => {
    expect(findLinkQuery('參考 [[')).toBe('')
    expect(findLinkQuery('參考 [[會議')).toBe('會議')
    expect(findLinkQuery('[[done]] after')).toBeNull()
    expect(findLinkQuery('no link')).toBeNull()
    expect(findLinkQuery(`[[${'x'.repeat(41)}`)).toBeNull()
  })
})
