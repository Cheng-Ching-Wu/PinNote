// @vitest-environment happy-dom
import { Editor } from '@tiptap/core'
import { afterEach, describe, expect, it } from 'vitest'

import { createNoteExtensions } from '@/lib/noteEditor'
import { isLossyLoad } from '@/utils/markdown'

let editor: Editor | undefined

function load(markdown: string, resolveImageSrc?: (src: string | null) => string | null) {
  editor = new Editor({
    extensions: createNoteExtensions({ resolveImageSrc }),
    content: markdown,
    contentType: 'markdown',
  })
  return editor
}

afterEach(() => editor?.destroy())

const types = (e: Editor) => (e.getJSON().content ?? []).map((n) => n.type)

describe('tables', () => {
  const md = ['前言', '', '| 欄位 | 說明 |', '| --- | :---: |', '| id | **主鍵** |', '| name | 名稱 |', '', '結尾'].join('\n')

  it('loads GFM tables without losing content', () => {
    const e = load(md)
    expect(types(e)).toEqual(['paragraph', 'table', 'paragraph'])
    expect(isLossyLoad(md, e.getText())).toBe(false)
  })

  it('round-trips tables (including alignment and inline marks)', () => {
    const saved = load(md).getMarkdown()
    expect(saved).toContain('**主鍵**')
    const json = JSON.stringify(editor!.getJSON())
    editor!.destroy()
    expect(JSON.stringify(load(saved).getJSON())).toBe(json)
  })

  it('inserts a table that serializes as markdown', () => {
    const e = load('')
    e.chain().insertTable({ rows: 2, cols: 2, withHeaderRow: true }).run()
    expect(e.getMarkdown()).toMatch(/\|.*\|\n\|\s*-+\s*\|\s*-+\s*\|/)
  })
})

describe('images', () => {
  const md = '截圖如下 ![登入畫面](images/abc123.png) 以及 ![logo](https://example.com/logo.png)'

  it('keeps the stored src in markdown but renders a resolved URL', () => {
    const e = load(md, (src) => (src?.startsWith('images/') ? `asset://${src}` : src))
    expect(e.getMarkdown()).toContain('![登入畫面](images/abc123.png)')
    expect(e.getMarkdown()).toContain('![logo](https://example.com/logo.png)')

    const html = e.getHTML()
    expect(html).toContain('src="asset://images/abc123.png"')
    expect(html).toContain('data-src="images/abc123.png"')
    expect(isLossyLoad(md, e.getText())).toBe(false)
  })

  it('round-trips an inserted image', () => {
    const e = load('文字')
    e.chain().focus('end').insertContent({ type: 'image', attrs: { src: 'images/x.png', alt: '' } }).run()
    const saved = e.getMarkdown()
    expect(saved).toContain('![](images/x.png)')
    e.destroy()
    expect(load(saved).getMarkdown()).toBe(saved)
  })
})

describe('lossy load detection', () => {
  it('flags raw HTML, whose markup would be dropped on save', () => {
    const md = '前言\n\n<div class="box">重要的 HTML 內容</div>\n\n結尾'
    const e = load(md)
    expect(isLossyLoad(md, e.getText())).toBe(true)
  })

  it('flags text that disappears entirely', () => {
    expect(isLossyLoad('前言\n\n| a | b |\n| - | - |\n| 1 | 2 |', '前言')).toBe(true)
  })

  it('does not flag ordinary notes', () => {
    for (const md of [
      'RTPMS\n碼頭 API\n航行時間 API',
      '# 標題\n\n- [x] 完成\n- [ ] 待辦\n\n> 引用 `code` [連結](https://a.b) a &amp; b',
      '```js\n# not a heading\nconst a = 1\n```',
    ]) {
      const e = load(md)
      expect(isLossyLoad(md, e.getText()), md).toBe(false)
      e.destroy()
      editor = undefined
    }
  })
})
