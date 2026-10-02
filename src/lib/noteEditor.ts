import type { Editor, Extensions } from '@tiptap/core'
import Highlight from '@tiptap/extension-highlight'
import { TaskItem, TaskList } from '@tiptap/extension-list'
import { Placeholder } from '@tiptap/extensions'
import { Markdown } from '@tiptap/markdown'
import StarterKit from '@tiptap/starter-kit'

/**
 * Note content is edited WYSIWYG but stored as Markdown text in `notes.content`,
 * so search, JSON export and older plain-text notes keep working unchanged.
 *
 * Only styles Markdown can represent are offered (underline is stored as `++text++`,
 * highlight as `==text==`).
 */
export function createNoteExtensions(placeholder: string): Extensions {
  return [
    StarterKit.configure({
      heading: { levels: [1, 2, 3] },
      // Links open in the system browser via Ctrl+click (see NoteEditor.vue);
      // a plain click must never navigate the note's webview away.
      link: { openOnClick: false, autolink: true, defaultProtocol: 'https' },
    }),
    TaskList,
    TaskItem.configure({ nested: true }),
    Highlight,
    Placeholder.configure({ placeholder }),
    Markdown.configure({
      // `breaks`: single newlines are line breaks, so plain-text notes written
      // before Markdown support keep their line structure.
      markedOptions: { gfm: true, breaks: true },
    }),
  ]
}

/**
 * Like `toggleCodeBlock`, but a selection spanning several blocks becomes ONE
 * code block (one line per block) instead of a separate code block per paragraph.
 */
export function toggleMergedCodeBlock(editor: Editor): boolean {
  const { $from, $to } = editor.state.selection
  if (editor.isActive('codeBlock') || $from.sameParent($to)) {
    return editor.chain().focus().toggleCodeBlock().run()
  }

  // Replace the whole top-level blocks the selection touches, so no text is lost.
  const doc = editor.state.doc
  const start = $from.depth >= 1 ? $from.before(1) : 0
  const end = $to.depth >= 1 ? $to.after(1) : doc.content.size

  const lines: string[] = []
  doc.nodesBetween(start, end, (node) => {
    if (!node.isTextblock) return true
    lines.push(node.textContent)
    return false
  })
  const text = lines.join('\n')

  return editor
    .chain()
    .focus()
    .insertContentAt({ from: start, to: end }, { type: 'codeBlock', content: text ? [{ type: 'text', text }] : [] })
    .run()
}
