<script setup lang="ts">
import { computed, ref } from 'vue'
import { EditorContent, useEditor } from '@tiptap/vue-3'

import NoteToolbar from '@/components/note/NoteToolbar.vue'
import { createNoteExtensions } from '@/lib/noteEditor'
import { cn } from '@/lib/utils'
import { settingsService } from '@/services/settingsService'

const props = defineProps<{
  /** Markdown; only read once — the editor owns the content afterwards. */
  initialContent: string
  autofocus?: boolean
}>()

const emit = defineEmits<{ change: [markdown: string] }>()

const hovered = ref(false)

const editor = useEditor({
  content: props.initialContent,
  contentType: 'markdown',
  autofocus: props.autofocus ? 'end' : false,
  extensions: createNoteExtensions('寫點什麼…'),
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
  onUpdate: ({ editor }) => emit('change', editor.getMarkdown()),
})

/** Formatting bar is shown while hovering the note or typing in it. */
const showToolbar = computed(() => hovered.value || !!editor.value?.isFocused)

/** Clicking the empty area below the text should start typing at the end. */
function focusFromBlank(event: MouseEvent) {
  if (event.target === event.currentTarget) editor.value?.commands.focus('end')
}

defineExpose({
  focusEnd: () => editor.value?.commands.focus('end'),
})
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col" @mouseenter="hovered = true" @mouseleave="hovered = false">
    <EditorContent
      :editor="editor"
      class="min-h-0 flex-1 cursor-text overflow-y-auto px-3 py-2"
      @mousedown="focusFromBlank"
    />
    <NoteToolbar
      v-if="editor"
      :editor="editor"
      :class="cn('transition-opacity', showToolbar ? 'opacity-100' : 'pointer-events-none opacity-0')"
    />
  </div>
</template>
