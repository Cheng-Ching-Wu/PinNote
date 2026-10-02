// @vitest-environment happy-dom
import { Editor } from '@tiptap/core'
import { afterEach, describe, expect, it } from 'vitest'

import { createNoteExtensions, toggleMergedCodeBlock } from '@/lib/noteEditor'

let editor: Editor | undefined

function load(markdown: string) {
  editor = new Editor({
    extensions: createNoteExtensions(),
    content: markdown,
    contentType: 'markdown',
  })
  return editor
}

afterEach(() => editor?.destroy())

describe('note editor markdown round-trip', () => {
  it('keeps plain-text notes written before markdown support line by line', () => {
    const e = load('RTPMS\n碼頭 API\n航行時間 API')
    expect(e.getText({ blockSeparator: '\n' }).split('\n')).toEqual(['RTPMS', '碼頭 API', '航行時間 API'])

    // Saving and loading again must be stable.
    const saved = e.getMarkdown()
    e.destroy()
    expect(load(saved).getMarkdown()).toBe(saved)
  })

  it('parses and serializes every toolbar style', () => {
    const md = [
      '# 標題',
      '',
      '**粗體** *斜體* ++底線++ ~~刪除~~ ==標記== `code`',
      '',
      '- 項目',
      '',
      '1. 第一',
      '',
      '- [ ] 待辦',
      '- [x] 完成',
      '',
      '> 引用',
    ].join('\n')

    const e = load(md)
    const json = JSON.stringify(e.getJSON())
    for (const type of ['heading', 'bold', 'italic', 'underline', 'strike', 'highlight', 'code', 'bulletList', 'orderedList', 'taskList', 'blockquote']) {
      expect(json, type).toContain(`"type":"${type}"`)
    }
    expect(json).toContain('"checked":true')

    const saved = e.getMarkdown()
    e.destroy()
    expect(JSON.stringify(load(saved).getJSON())).toBe(json)
  })

  it('stores toolbar formatting as standard markdown', () => {
    const e = load('')
    e.chain().toggleTaskList().insertContent('買牛奶').run()
    expect(e.getMarkdown()).toContain('- [ ] 買牛奶')

    e.commands.setContent('')
    e.chain().toggleHeading({ level: 2 }).insertContent('會議').run()
    expect(e.getMarkdown().trim()).toBe('## 會議')
  })
})

/** Selects the first occurrence of `text` in the document. */
function select(e: Editor, text: string) {
  let from = -1
  e.state.doc.descendants((node, pos) => {
    if (from === -1 && node.isText && node.text!.includes(text)) from = pos + node.text!.indexOf(text)
  })
  e.commands.setTextSelection({ from, to: from + text.length })
}

describe('code formatting', () => {
  it('inline code only wraps the selected text', () => {
    const e = load('呼叫 getNotes 取得資料')
    select(e, 'getNotes')
    e.chain().toggleCode().run()
    expect(e.getMarkdown().trim()).toBe('呼叫 `getNotes` 取得資料')
  })

  it('code block on a multi-paragraph selection makes a single block', () => {
    const e = load('前言\n\nconst a = 1\n\nconst b = 2\n\n結尾')
    e.commands.setTextSelection({ from: 0, to: 0 })
    select(e, 'const a')
    const { from } = e.state.selection
    select(e, '= 2')
    e.commands.setTextSelection({ from, to: e.state.selection.to })

    toggleMergedCodeBlock(e)

    const blocks = e.getJSON().content!.map((n) => n.type)
    expect(blocks).toEqual(['paragraph', 'codeBlock', 'paragraph'])
    expect(e.getMarkdown()).toContain('```\nconst a = 1\nconst b = 2\n```')
  })

  it('code block on a cursor converts just that paragraph, and toggles back', () => {
    const e = load('第一段\n\n第二段')
    select(e, '第二')
    const types = () => e.getJSON().content!.map((n) => n.type)
    toggleMergedCodeBlock(e)
    // A trailing empty paragraph is added after a final code block so the caret can leave it.
    expect(types().slice(0, 2)).toEqual(['paragraph', 'codeBlock'])
    expect(e.state.doc.child(1).textContent).toBe('第二段')
    toggleMergedCodeBlock(e)
    expect(types()).not.toContain('codeBlock')
  })
})
