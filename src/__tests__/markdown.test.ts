import { describe, expect, it } from 'vitest'

import type { Note } from '@/models/note'
import { matchesQuery } from '@/stores/note'
import { markdownToPlainText } from '@/utils/markdown'

describe('markdownToPlainText', () => {
  it('strips block syntax but keeps lines', () => {
    const md = ['# API 待確認', '- RTPMS', '1. 碼頭 API', '- [ ] 航行時間', '- [x] 油駁船', '> 引用'].join('\n')
    expect(markdownToPlainText(md).split('\n')).toEqual(['API 待確認', 'RTPMS', '碼頭 API', '航行時間', '油駁船', '引用'])
  })

  it('strips inline marks, links and escapes', () => {
    expect(markdownToPlainText('**粗** *斜* ~~刪~~ ==標記== ++底線++ `code`')).toBe('粗 斜 刪 標記 底線 code')
    expect(markdownToPlainText('看 [文件](https://example.com) 1\\. 不是清單')).toBe('看 文件 1. 不是清單')
  })

  it('leaves snake_case and plain text alone', () => {
    expect(markdownToPlainText('rtpms_api_v2\n第二行')).toBe('rtpms_api_v2\n第二行')
  })

  it('removes hard-break spaces and empty-paragraph markers', () => {
    expect(markdownToPlainText('第一行  \n第二行\n\n&nbsp;')).toBe('第一行\n第二行\n\n')
  })
})

describe('matchesQuery with markdown', () => {
  it('matches text across formatting', () => {
    const note = { title: '', content: '**碼頭** API' } as Note
    expect(matchesQuery(note, '碼頭 API')).toBe(true)
  })
})
