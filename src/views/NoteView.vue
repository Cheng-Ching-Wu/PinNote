<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, useTemplateRef } from 'vue'

import NoteHeader from '@/components/note/NoteHeader.vue'
import { useNoteDraft } from '@/composables/useNoteDraft'
import { useNoteWindow } from '@/composables/useNoteWindow'
import { NOTE_COLOR_CLASSES } from '@/constants/noteColors'
import { CONTENT_SAVE_DELAY } from '@/constants/window'
import { cn } from '@/lib/utils'
import { windowService } from '@/services/windowService'
import { useNoteStore } from '@/stores/note'

const props = defineProps<{ id: string }>()

const store = useNoteStore()
const { note, toggleAlwaysOnTop, toggleCollapsed, setColor } = useNoteWindow(props.id)
const { title, content, init, flush } = useNoteDraft(props.id, CONTENT_SAVE_DELAY)

const contentEl = useTemplateRef<HTMLTextAreaElement>('contentEl')

function focusContent() {
  const el = contentEl.value
  if (!el) return
  el.focus()
  el.setSelectionRange(el.value.length, el.value.length)
}

/** Save pending typing before the window loses focus or goes away. */
const flushOnBlur = () => void flush()

onMounted(async () => {
  const loaded = await store.loadNote(props.id)
  if (!loaded || loaded.status !== 'active') {
    await windowService.closeNoteWindow(props.id)
    return
  }
  init(loaded)
  window.addEventListener('blur', flushOnBlur)

  // A brand-new (empty) note is ready for typing straight away.
  if (!loaded.collapsed && !loaded.title && !loaded.content) {
    await nextTick()
    focusContent()
  }
})

onBeforeUnmount(() => window.removeEventListener('blur', flushOnBlur))

async function close() {
  await flush()
  await store.closeNoteWindow(props.id)
}

async function archive() {
  await flush()
  await store.archiveNote(props.id)
}

async function remove() {
  await flush()
  await store.deleteNote(props.id)
}
</script>

<template>
  <div
    v-if="note"
    :class="
      cn(NOTE_COLOR_CLASSES[note.color], 'group/note flex h-screen flex-col overflow-hidden bg-note-body text-note-ink')
    "
  >
    <NoteHeader
      v-model:title="title"
      :color="note.color"
      :collapsed="note.collapsed"
      :always-on-top="note.alwaysOnTop"
      @title-enter="focusContent"
      @toggle-pin="toggleAlwaysOnTop"
      @toggle-collapse="toggleCollapsed"
      @close="close"
      @set-color="setColor"
      @new-note="store.createNote()"
      @open-main="windowService.showMainWindow()"
      @archive="archive"
      @delete="remove"
    />
    <textarea
      v-show="!note.collapsed"
      ref="contentEl"
      v-model="content"
      placeholder="寫點什麼…"
      spellcheck="false"
      class="min-h-0 flex-1 resize-none bg-transparent px-3 py-2 text-[14px] leading-relaxed outline-none placeholder:text-note-ink/35"
    />
  </div>
</template>
