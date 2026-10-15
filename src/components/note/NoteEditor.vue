<script setup lang="ts">
import { computed, reactive, ref, useTemplateRef } from 'vue'
import type { Editor } from '@tiptap/core'
import { EditorContent, useEditor } from '@tiptap/vue-3'
import { Archive, Plus, TriangleAlert } from 'lucide-vue-next'

import NoteToolbar from '@/components/note/NoteToolbar.vue'
import { t } from '@/i18n'
import { createNoteExtensions } from '@/lib/noteEditor'
import { cn } from '@/lib/utils'
import { IMAGE_MIME_TYPES, imageService } from '@/services/imageService'
import { settingsService } from '@/services/settingsService'
import { isLossyLoad } from '@/utils/markdown'
import { findLinkQuery, type LinkTarget, noteIdFromHref, noteLinkHref } from '@/utils/noteLinks'

const props = defineProps<{
  /** Markdown; only read once — the editor owns the content afterwards. */
  initialContent: string
  autofocus?: boolean
  /** Notes matching the text typed after `[[` (reactive: re-run as notes load). */
  findLinkTargets?: (query: string) => LinkTarget[]
  /** Creates a note with this title for a `[[` link to it; resolves to its id. */
  createLinkTarget?: (title: string) => Promise<string | undefined>
}>()

const emit = defineEmits<{
  change: [markdown: string]
  /** Ctrl+click on a link to another note. */
  'open-note': [id: string]
}>()

const hovered = ref(false)
/** The stored Markdown has content the editor can't show; editing would drop it. */
const readOnlyGuard = ref(false)
const message = ref<string | null>(null)

/** Pasted screenshots are called "image.png" — not a useful alt text. */
function altFromFileName(name: string) {
  const base = name.replace(/\.[^.]+$/, '')
  return /^image$/i.test(base) ? '' : base
}

async function insertImages(target: Editor, files: File[], pos?: number) {
  message.value = null
  for (const file of files) {
    try {
      const src = await imageService.save(file)
      const image = { type: 'image', attrs: { src, alt: altFromFileName(file.name) } }
      if (pos === undefined) {
        target.chain().focus().insertContent(image).run()
      } else {
        target.chain().insertContentAt(pos, image).focus().run()
        pos += 1
      }
    } catch (error) {
      message.value = t('editor.imageFailed', { name: file.name, error: String(error) })
    }
  }
}

// ---- `[[` note links ----

const rootRef = useTemplateRef<HTMLElement>('rootRef')

/** The `[[query` being typed: document range to replace and where to show the list. */
const linkMenu = reactive({
  open: false,
  query: '',
  from: 0,
  to: 0,
  index: 0,
  /** Position inside the editor root (px). */
  left: 0,
  top: 0,
  /** Opens upwards when there isn't room below the cursor. */
  above: false,
})

const linkTargets = computed(() =>
  linkMenu.open && props.findLinkTargets ? props.findLinkTargets(linkMenu.query.trim()) : [],
)
/** Typed text that isn't an existing note's heading can become a new note. */
const linkNewTitle = computed(() => {
  const title = linkMenu.query.trim()
  if (!title || !props.createLinkTarget) return null
  return linkTargets.value.some((t) => t.heading.toLowerCase() === title.toLowerCase()) ? null : title
})
const linkRowCount = computed(() => linkTargets.value.length + (linkNewTitle.value ? 1 : 0))

const LINK_MENU_HEIGHT = 180
const LINK_MENU_WIDTH = 220

function updateLinkMenu(target: Editor) {
  const { selection } = target.state
  const { $from } = selection
  const query =
    props.findLinkTargets && selection.empty && !$from.parent.type.spec.code
      ? findLinkQuery($from.parent.textBetween(0, $from.parentOffset, undefined, '\ufffc'))
      : null
  const root = rootRef.value
  if (query === null || !root) {
    linkMenu.open = false
    return
  }

  const from = $from.pos - query.length - 2
  const coords = target.view.coordsAtPos(from)
  const rect = root.getBoundingClientRect()
  const below = coords.bottom - rect.top + 4
  if (!linkMenu.open || linkMenu.query !== query) linkMenu.index = 0
  Object.assign(linkMenu, {
    open: true,
    query,
    from,
    to: $from.pos,
    left: Math.max(4, Math.min(coords.left - rect.left, rect.width - LINK_MENU_WIDTH - 4)),
    above: below + LINK_MENU_HEIGHT > rect.height && coords.top - rect.top > LINK_MENU_HEIGHT / 2,
    top: below,
  })
  if (linkMenu.above) linkMenu.top = rect.bottom - coords.top + 4
}

/** Replaces `[[query` with a link chip titled like the note, plus a space to keep typing. */
function insertLink(id: string, text: string) {
  const target = editor.value
  if (!target) return
  const { from, to } = linkMenu
  linkMenu.open = false
  target
    .chain()
    .focus()
    .insertContentAt({ from, to }, [
      { type: 'text', text, marks: [{ type: 'link', attrs: { href: noteLinkHref(id) } }] },
      { type: 'text', text: ' ' },
    ])
    .run()
}

async function chooseLink(index: number) {
  const chosen = linkTargets.value[index]
  if (chosen) return insertLink(chosen.id, chosen.heading)
  const title = linkNewTitle.value
  if (title === null || !props.createLinkTarget) return
  const id = await props.createLinkTarget(title)
  if (id) insertLink(id, title)
}

/** Arrow keys / Enter / Tab / Escape drive the list while it is open. */
function handleLinkMenuKey(event: KeyboardEvent): boolean {
  if (!linkMenu.open || event.isComposing) return false
  const count = linkRowCount.value
  switch (event.key) {
    case 'ArrowDown':
      if (count) linkMenu.index = (linkMenu.index + 1) % count
      return true
    case 'ArrowUp':
      if (count) linkMenu.index = (linkMenu.index - 1 + count) % count
      return true
    case 'Enter':
    case 'Tab':
      if (!count) return false
      void chooseLink(linkMenu.index)
      return true
    case 'Escape':
      linkMenu.open = false
      return true
    default:
      return false
  }
}

const editor = useEditor({
  content: props.initialContent,
  contentType: 'markdown',
  autofocus: props.autofocus ? 'end' : false,
  extensions: createNoteExtensions({
    placeholder: t('editor.placeholder'),
    resolveImageSrc: imageService.resolveSrc,
    imageMimeTypes: IMAGE_MIME_TYPES,
    onImageFiles: (target, files, pos) => void insertImages(target, files, pos),
  }),
  editorProps: {
    attributes: { class: 'note-editor', spellcheck: 'false' },
    handleKeyDown: (_view, event) => handleLinkMenuKey(event),
    handleClick(_view, _pos, event) {
      const link = (event.target as HTMLElement | null)?.closest('a')
      if (!link) return false
      event.preventDefault()
      if (!event.ctrlKey && !event.metaKey) return true
      const noteId = noteIdFromHref(link.getAttribute('href'))
      if (noteId) emit('open-note', noteId)
      else void settingsService.openExternal(link.href).catch(console.error)
      return true
    },
  },
  onCreate: ({ editor: created }) => {
    if (isLossyLoad(props.initialContent, created.getText())) {
      readOnlyGuard.value = true
      created.setEditable(false)
    }
  },
  onUpdate: ({ editor: updated }) => {
    emit('change', updated.getMarkdown())
    updateLinkMenu(updated)
  },
  onSelectionUpdate: ({ editor: updated }) => updateLinkMenu(updated),
  onBlur: () => (linkMenu.open = false),
})

/** Formatting bar is shown while hovering the note or typing in it. */
const showToolbar = computed(() => !readOnlyGuard.value && (hovered.value || !!editor.value?.isFocused))

/** Clicking the empty area below the text should start typing at the end. */
function focusFromBlank(event: MouseEvent) {
  if (event.target === event.currentTarget && !readOnlyGuard.value) editor.value?.commands.focus('end')
}

async function copyOriginal() {
  await navigator.clipboard.writeText(props.initialContent)
  message.value = t('editor.copiedOriginal')
}

function editAnyway() {
  readOnlyGuard.value = false
  editor.value?.setEditable(true)
  editor.value?.commands.focus('end')
}

defineExpose({
  focusEnd: () => {
    if (!readOnlyGuard.value) editor.value?.commands.focus('end')
  },
  /** Short notice inside the note, e.g. when a linked note can't be opened. */
  showMessage: (text: string) => (message.value = text),
})
</script>

<template>
  <div
    ref="rootRef"
    class="relative flex min-h-0 flex-1 flex-col"
    @mouseenter="hovered = true"
    @mouseleave="hovered = false"
  >
    <div
      v-if="readOnlyGuard"
      class="mx-2 mt-2 flex gap-1.5 rounded border border-note-border bg-note-header/70 px-2 py-1.5 text-xs leading-relaxed"
    >
      <TriangleAlert class="mt-0.5 size-3.5 shrink-0" />
      <div>
        {{ t('editor.readOnly') }}
        <div class="mt-1 flex gap-3 font-medium">
          <button type="button" class="underline underline-offset-2" @click="copyOriginal">{{ t('editor.copyOriginal') }}</button>
          <button type="button" class="underline underline-offset-2" @click="editAnyway">{{ t('editor.editAnyway') }}</button>
        </div>
      </div>
    </div>
    <p v-if="message" class="mx-2 mt-2 rounded bg-note-ink/10 px-2 py-1 text-xs" @click="message = null">
      {{ message }}
    </p>

    <EditorContent
      :editor="editor"
      class="min-h-0 flex-1 cursor-text overflow-y-auto px-3 py-2"
      @mousedown="focusFromBlank"
    />
    <!-- `[[` link picker; mousedown.prevent keeps the editor focused (blur would close it). -->
    <ul
      v-if="linkMenu.open && linkRowCount > 0"
      class="absolute z-30 max-h-45 overflow-y-auto rounded-md border bg-card p-0.5 text-[12px] text-foreground shadow-md"
      :style="{
        left: `${linkMenu.left}px`,
        width: `${LINK_MENU_WIDTH}px`,
        ...(linkMenu.above ? { bottom: `${linkMenu.top}px` } : { top: `${linkMenu.top}px` }),
      }"
      @mousedown.prevent
    >
      <li
        v-for="(target, index) in linkTargets"
        :key="target.id"
        :class="cn('flex h-7 cursor-pointer items-center gap-1.5 rounded-sm px-2', index === linkMenu.index && 'bg-accent')"
        @mousemove="linkMenu.index = index"
        @click="chooseLink(index)"
      >
        <span class="truncate">{{ target.heading }}</span>
        <Archive v-if="target.archived" class="ml-auto size-3 shrink-0 text-muted-foreground" />
      </li>
      <li
        v-if="linkNewTitle"
        :class="
          cn(
            'flex h-7 cursor-pointer items-center gap-1.5 rounded-sm px-2 text-muted-foreground',
            linkMenu.index === linkTargets.length && 'bg-accent',
          )
        "
        @mousemove="linkMenu.index = linkTargets.length"
        @click="chooseLink(linkTargets.length)"
      >
        <Plus class="size-3 shrink-0" />
        <span class="truncate">{{ t('common.newNoteNamed', { title: linkNewTitle }) }}</span>
      </li>
    </ul>

    <NoteToolbar
      v-if="editor"
      :editor="editor"
      :class="cn('transition-opacity', showToolbar ? 'opacity-100' : 'pointer-events-none opacity-0')"
      @insert-images="(files) => editor && insertImages(editor, files)"
    />
  </div>
</template>
