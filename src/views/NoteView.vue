<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, useTemplateRef, watch } from 'vue'

import NoteEditor from '@/components/note/NoteEditor.vue'
import NoteHeader from '@/components/note/NoteHeader.vue'
import NoteHistory from '@/components/note/NoteHistory.vue'
import NoteShortcutsHelp from '@/components/note/NoteShortcutsHelp.vue'
import NoteTags from '@/components/note/NoteTags.vue'
import { useNoteDraft } from '@/composables/useNoteDraft'
import { useNoteTranslucency } from '@/composables/useNoteTranslucency'
import { useNoteWindow } from '@/composables/useNoteWindow'
import { NOTE_COLOR_CLASSES } from '@/constants/noteColors'
import { CONTENT_SAVE_DELAY } from '@/constants/window'
import { t } from '@/i18n'
import { cn } from '@/lib/utils'
import { syncService } from '@/services/syncService'
import { updateService } from '@/services/updateService'
import { windowService } from '@/services/windowService'
import type { Note } from '@/models/note'
import { useNoteStore } from '@/stores/note'
import type { LinkTarget } from '@/utils/noteLinks'
import { searchNotes } from '@/utils/search'

const props = defineProps<{ id: string }>()

const store = useNoteStore()
const { note, toggleAlwaysOnTop, toggleCollapsed, setColor } = useNoteWindow(props.id)
const { title, content, init, flush, hasPending } = useNoteDraft(props.id, CONTENT_SAVE_DELAY)
const { faded, onPointerEnter, onPointerLeave } = useNoteTranslucency(note)

const editorRef = useTemplateRef<InstanceType<typeof NoteEditor>>('editorRef')

/** The editor reads its content once, so it is only created after the draft is initialised. */
const ready = ref(false)
/** A brand-new (empty) note is ready for typing straight away. */
const autofocus = ref(false)

const tagEditing = ref(false)
const tagSuggestions = computed(() => store.allTags.map((tag) => tag.name))

/**
 * Note windows only load their own note; tag suggestions and `[[` links need every
 * note. Loaded once, on first use (later changes arrive through the sync events).
 */
function ensureAllNotes() {
  if (!store.loaded && !loadingAll) {
    loadingAll = true
    void store.loadNotes()
  }
}
let loadingAll = false

watch(tagEditing, (editing) => {
  if (editing) ensureAllNotes()
})

// ---- Links to other notes ----

function findLinkTargets(query: string): LinkTarget[] {
  ensureAllNotes()
  const others = store.notes.filter((other) => other.id !== props.id)
  return searchNotes(others, query, 6).map(({ note: target, heading }) => ({
    id: target.id,
    heading,
    archived: target.status === 'archived',
  }))
}

/** `[[新標題` → a new note that stays off the desktop until the link is opened. */
async function createLinkTarget(title: string) {
  return (await store.createNoteWithTitle(title, { open: false }))?.id
}

async function openLinkedNote(targetId: string) {
  if (!store.loaded) await store.loadNotes()
  const target = store.getById(targetId)
  if (!target) return editorRef.value?.showMessage(t('editor.linkNotFound'))
  if (target.status === 'deleted') return editorRef.value?.showMessage(t('editor.linkInTrash'))
  if (target.status === 'archived') await store.restoreNote(targetId)
  await store.openNoteWindow(targetId)
}

// ---- History ----

/** Panel shown in place of the editor (and tags); null shows the note itself. */
const panel = ref<'history' | 'shortcuts' | null>(null)
/** Bumped to rebuild the editor after a restore: it only reads its content once. */
const editorKey = ref(0)

async function openPanel(name: 'history' | 'shortcuts') {
  if (name === 'history') await flush() // so the current text is what gets kept as a version
  if (note.value?.collapsed) await toggleCollapsed()
  tagEditing.value = false
  panel.value = name
}

function onRestored(restored: Note) {
  init(restored)
  editorKey.value++
  panel.value = null
}

async function editTags() {
  if (note.value?.collapsed) await toggleCollapsed()
  tagEditing.value = true
}

function focusContent() {
  editorRef.value?.focusEnd()
}

/** Save pending typing before the window loses focus or goes away. */
const flushOnBlur = () => void flush()

/**
 * Another device changed this note's text. The editor only reads its content once,
 * so it is rebuilt with the new text. If there is unsaved typing, that typing wins
 * (it is newer) and the other device's text is kept as a revision instead.
 */
async function onSynced(ids: string[]) {
  const incoming = store.getById(props.id)
  if (!ids.includes(props.id) || !incoming || !ready.value) return
  if (hasPending()) {
    await store.addRevision(props.id, incoming.title, incoming.content)
    await flush()
    return
  }
  init(incoming)
  editorKey.value++
  editorRef.value?.showMessage(t('editor.syncedFromOtherDevice'))
}
let stopSyncListener: (() => void) | undefined
let stopFlushListener: (() => void) | undefined

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
  stopSyncListener = await syncService.onNotesSynced((ids) => void onSynced(ids))
  // An update is about to close the app.
  stopFlushListener = await updateService.onFlushRequested(flushOnBlur)
})

onBeforeUnmount(() => {
  window.removeEventListener('blur', flushOnBlur)
  stopSyncListener?.()
  stopFlushListener?.()
})

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
      @edit-tags="editTags"
      @history="openPanel('history')"
      @shortcuts="openPanel('shortcuts')"
      @new-note="store.createNote()"
      @open-main="windowService.showMainWindow()"
      @archive="archive"
      @delete="remove"
    />
    <template v-if="!note.collapsed">
      <NoteHistory v-if="panel === 'history'" :note-id="id" @close="panel = null" @restored="onRestored" />
      <NoteShortcutsHelp v-else-if="panel === 'shortcuts'" @close="panel = null" />
    </template>
    <NoteEditor
      v-if="ready"
      v-show="!note.collapsed && !panel"
      ref="editorRef"
      :key="editorKey"
      :initial-content="content"
      :autofocus="autofocus"
      :find-link-targets="findLinkTargets"
      :create-link-target="createLinkTarget"
      @change="content = $event"
      @open-note="openLinkedNote"
    />
    <NoteTags
      v-if="ready && !note.collapsed && !panel"
      v-model:editing="tagEditing"
      :tags="note.tags"
      :suggestions="tagSuggestions"
      @change="store.setTags(id, $event)"
    />
  </div>
</template>
