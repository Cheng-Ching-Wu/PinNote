<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Editor } from '@tiptap/core'
import { EditorContent, useEditor } from '@tiptap/vue-3'
import { TriangleAlert } from 'lucide-vue-next'

import NoteToolbar from '@/components/note/NoteToolbar.vue'
import { createNoteExtensions } from '@/lib/noteEditor'
import { cn } from '@/lib/utils'
import { IMAGE_MIME_TYPES, imageService } from '@/services/imageService'
import { settingsService } from '@/services/settingsService'
import { isLossyLoad } from '@/utils/markdown'

const props = defineProps<{
  /** Markdown; only read once — the editor owns the content afterwards. */
  initialContent: string
  autofocus?: boolean
}>()

const emit = defineEmits<{ change: [markdown: string] }>()

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
      message.value = `無法加入圖片「${file.name}」：${String(error)}`
    }
  }
}

const editor = useEditor({
  content: props.initialContent,
  contentType: 'markdown',
  autofocus: props.autofocus ? 'end' : false,
  extensions: createNoteExtensions({
    placeholder: '寫點什麼…',
    resolveImageSrc: imageService.resolveSrc,
    imageMimeTypes: IMAGE_MIME_TYPES,
    onImageFiles: (target, files, pos) => void insertImages(target, files, pos),
  }),
  editorProps: {
    attributes: { class: 'note-editor', spellcheck: 'false' },
    handleClick(_view, _pos, event) {
      const link = (event.target as HTMLElement | null)?.closest('a')
      if (!link) return false
      event.preventDefault()
      if (event.ctrlKey || event.metaKey) void settingsService.openExternal(link.href).catch(console.error)
      return true
    },
  },
  onCreate: ({ editor: created }) => {
    if (isLossyLoad(props.initialContent, created.getText())) {
      readOnlyGuard.value = true
      created.setEditable(false)
    }
  },
  onUpdate: ({ editor: updated }) => emit('change', updated.getMarkdown()),
})

/** Formatting bar is shown while hovering the note or typing in it. */
const showToolbar = computed(() => !readOnlyGuard.value && (hovered.value || !!editor.value?.isFocused))

/** Clicking the empty area below the text should start typing at the end. */
function focusFromBlank(event: MouseEvent) {
  if (event.target === event.currentTarget && !readOnlyGuard.value) editor.value?.commands.focus('end')
}

async function copyOriginal() {
  await navigator.clipboard.writeText(props.initialContent)
  message.value = '已複製原始 Markdown'
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
})
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col" @mouseenter="hovered = true" @mouseleave="hovered = false">
    <div
      v-if="readOnlyGuard"
      class="mx-2 mt-2 flex gap-1.5 rounded border border-note-border bg-note-header/70 px-2 py-1.5 text-xs leading-relaxed"
    >
      <TriangleAlert class="mt-0.5 size-3.5 shrink-0" />
      <div>
        這張便籤含有無法在編輯器顯示的內容（例如 HTML），為避免存檔時遺失，目前為唯讀。
        <div class="mt-1 flex gap-3 font-medium">
          <button type="button" class="underline underline-offset-2" @click="copyOriginal">複製原始內容</button>
          <button type="button" class="underline underline-offset-2" @click="editAnyway">仍要編輯</button>
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
    <NoteToolbar
      v-if="editor"
      :editor="editor"
      :class="cn('transition-opacity', showToolbar ? 'opacity-100' : 'pointer-events-none opacity-0')"
      @insert-images="(files) => editor && insertImages(editor, files)"
    />
  </div>
</template>
