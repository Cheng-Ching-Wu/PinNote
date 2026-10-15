import type { Editor, Extensions } from '@tiptap/core'
import FileHandler from '@tiptap/extension-file-handler'
import Highlight from '@tiptap/extension-highlight'
import Image from '@tiptap/extension-image'
import { TaskItem, TaskList } from '@tiptap/extension-list'
import { TableKit } from '@tiptap/extension-table'
import { Placeholder } from '@tiptap/extensions'
import { Markdown } from '@tiptap/markdown'
import StarterKit from '@tiptap/starter-kit'

import { t } from '@/i18n'
import { NOTE_LINK_SCHEME } from '@/utils/noteLinks'

export interface NoteEditorOptions {
  placeholder?: string
  /** Maps a stored `src` (e.g. `images/x.png`) to a URL the webview can load. */
  resolveImageSrc?: (src: string | null) => string | null
  /** Image files pasted or dropped into the editor (`pos` is set for drops). */
  onImageFiles?: (editor: Editor, files: File[], pos?: number) => void
  imageMimeTypes?: string[]
}

/**
 * Image node whose Markdown `src` stays as written (`images/<file>` or a web URL)
 * while the rendered `<img>` gets a loadable URL. `data-src` keeps the original
 * so copy / paste inside the editor round-trips correctly.
 */
function createNoteImage(resolve: (src: string | null) => string | null) {
  return Image.extend({
    addAttributes() {
      return {
        ...this.parent?.(),
        src: {
          default: null,
          parseHTML: (element: HTMLElement) => element.getAttribute('data-src') ?? element.getAttribute('src'),
          renderHTML: (attributes: { src?: string | null }) => ({
            src: resolve(attributes.src ?? null),
            'data-src': attributes.src,
          }),
        },
      }
    },
  }).configure({ inline: true, allowBase64: false })
}

/**
 * Note content is edited WYSIWYG but stored as Markdown text in `notes.content`,
 * so search, JSON export and older plain-text notes keep working unchanged.
 *
 * Only styles Markdown can represent are offered (underline is stored as `++text++`,
 * highlight as `==text==`). Tables are GFM tables; images are `![alt](src)`.
 */
export function createNoteExtensions(options: NoteEditorOptions = {}): Extensions {
  const extensions: Extensions = [
    StarterKit.configure({
      heading: { levels: [1, 2, 3] },
      // Links open via Ctrl+click (see NoteEditor.vue): web links in the system browser,
      // `pinnote://note/<id>` links in their note window. A plain click must never
      // navigate the note's webview away.
      link: {
        openOnClick: false,
        autolink: true,
        defaultProtocol: 'https',
        protocols: [NOTE_LINK_SCHEME],
        HTMLAttributes: { rel: 'noopener noreferrer nofollow', title: t('note.linkTitle') },
      },
    }),
    TaskList,
    TaskItem.configure({ nested: true }),
    Highlight,
    // Column widths can't be stored in Markdown, so no resizing.
    TableKit.configure({ table: { resizable: false } }),
    createNoteImage(options.resolveImageSrc ?? ((src) => src)),
    Placeholder.configure({ placeholder: options.placeholder ?? '' }),
    Markdown.configure({
      // `breaks`: single newlines are line breaks, so plain-text notes written
      // before Markdown support keep their line structure.
      markedOptions: { gfm: true, breaks: true },
    }),
  ]

  const onImageFiles = options.onImageFiles
  if (onImageFiles) {
    extensions.push(
      FileHandler.configure({
        allowedMimeTypes: options.imageMimeTypes,
        onPaste: (editor, files) => onImageFiles(editor, files),
        onDrop: (editor, files, pos) => onImageFiles(editor, files, pos),
      }),
    )
  }
  return extensions
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
