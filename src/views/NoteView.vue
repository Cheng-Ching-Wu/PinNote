<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, useTemplateRef } from 'vue'

import NoteEditor from '@/components/note/NoteEditor.vue'
import NoteHeader from '@/components/note/NoteHeader.vue'
import { useNoteDraft } from '@/composables/useNoteDraft'
import { useNoteTranslucency } from '@/composables/useNoteTranslucency'
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
const { faded, onPointerEnter, onPointerLeave } = useNoteTranslucency(note)

const editorRef = useTemplateRef<InstanceType<typeof NoteEditor>>('editorRef')

/** The editor reads its content once, so it is only created after the draft is initialised. */
const ready = ref(false)
/** A brand-new (empty) note is ready for typing straight away. */
const autofocus = ref(false)

function focusContent() {
  editorRef.value?.focusEnd()
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
  autofocus.value = !loaded.collapsed && !loaded.title && !loaded.content
  ready.value = true
  window.addEventListener('blur', flushOnBlur)
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
      cn(
        NOTE_COLOR_CLASSES[note.color],
        'group/note flex h-screen flex-col overflow-hidden bg-note-body text-note-ink transition-opacity duration-200',
        faded ? 'opacity-55' : 'opacity-100',
      )
    "
    @pointerenter="onPointerEnter"
    @pointerleave="onPointerLeave"
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
    <NoteEditor
      v-if="ready"
      v-show="!note.collapsed"
      ref="editorRef"
      :initial-content="content"
      :autofocus="autofocus"
      @change="content = $event"
    />
  </div>
</template>
