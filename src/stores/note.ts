import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import type { ArrangeAnchor, ArrangeLayout } from '@/constants/arrange'
import { type MessageKey, t } from '@/i18n'
import type { Note, NoteColor, NoteUpdate } from '@/models/note'
import { noteRepository } from '@/repositories/noteRepository'
import { imageService } from '@/services/imageService'
import { windowService } from '@/services/windowService'
import { formatBytes } from '@/utils/format'
import { plainTextOf } from '@/utils/noteText'
import { collectTags, hasTag, normalizeTags, parseTagQuery, tagKey } from '@/utils/tags'

const byUpdatedDesc = (a: Note, b: Note) => b.updatedAt.localeCompare(a.updatedAt)
const byDeletedDesc = (a: Note, b: Note) => (b.deletedAt ?? '').localeCompare(a.deletedAt ?? '')

/**
 * Matches title and content; `#term` words must each match (part of) one of the note's tags,
 * e.g. `#工作 會議`.
 */
export function matchesQuery(note: Note, query: string): boolean {
  const { tags, text } = parseTagQuery(query)
  const tagsMatch = tags.every((term) => note.tags.some((tag) => tagKey(tag).includes(tagKey(term))))
  if (!tagsMatch) return false
  const q = text.toLowerCase()
  if (!q) return true
  return note.title.toLowerCase().includes(q) || plainTextOf(note.content).toLowerCase().includes(q)
}

export const useNoteStore = defineStore('note', () => {
  const notes = ref<Note[]>([])
  const loaded = ref(false)
  const lastError = ref<string | null>(null)
  /** Short success message for the manager window (export / import / show all). */
  const notice = ref<string | null>(null)

  const activeNotes = computed(() => notes.value.filter((n) => n.status === 'active').sort(byUpdatedDesc))
  const archivedNotes = computed(() => notes.value.filter((n) => n.status === 'archived').sort(byUpdatedDesc))
  const deletedNotes = computed(() => notes.value.filter((n) => n.status === 'deleted').sort(byDeletedDesc))
  /** Tags of notes that aren't in the trash, for suggestions, filters and the tag manager. */
  const allTags = computed(() => collectTags(notes.value.filter((n) => n.status !== 'deleted')))

  function getById(id: string): Note | undefined {
    return notes.value.find((n) => n.id === id)
  }

  function upsert(note: Note) {
    const index = notes.value.findIndex((n) => n.id === note.id)
    if (index === -1) notes.value.push(note)
    else notes.value[index] = note
  }

  function remove(id: string) {
    notes.value = notes.value.filter((n) => n.id !== id)
  }

  /** Runs a repository call; errors are recorded instead of crashing the UI. */
  async function run<T>(label: MessageKey, task: () => Promise<T>): Promise<T | undefined> {
    try {
      return await task()
    } catch (error) {
      lastError.value = `${t(label)}: ${String(error)}`
      console.error(`[note-store] ${label}`, error)
      return undefined
    }
  }

  async function loadNotes() {
    const list = await run('errors.loadNotes', () => noteRepository.getNotes())
    if (list) {
      notes.value = list
      loaded.value = true
    }
  }

  async function loadNote(id: string) {
    const note = await run('errors.loadNotes', () => noteRepository.getNote(id))
    if (note) upsert(note)
    return note
  }

  /** Creates a note, opens its window and lets it take focus. */
  async function createNote() {
    const note = await run('errors.createNote', () => noteRepository.createNote())
    if (!note) return undefined
    upsert(note)
    await openNoteWindow(note.id)
    return note
  }

  /**
   * A new note with a title (quick search, `[[` links). The title is saved before the
   * window loads it. With `open: false` the note stays off the desktop.
   */
  async function createNoteWithTitle(title: string, { open = true } = {}) {
    const created = await run('errors.createNote', () => noteRepository.createNote())
    if (!created) return undefined
    const note = (await run('errors.createNote', () => noteRepository.updateNote(created.id, { title }))) ?? created
    upsert(note)
    if (open) await openNoteWindow(note.id)
    else await closeNoteWindow(note.id) // records `isOpen = false` (no window exists)
    return note
  }

  function addRevision(id: string, title: string, content: string) {
    return run('errors.addRevision', () => noteRepository.addRevision(id, title, content))
  }

  function loadRevisions(id: string) {
    return run('errors.loadRevisions', () => noteRepository.getRevisions(id))
  }

  async function restoreRevision(id: string, revisionId: number) {
    const note = await run('errors.restoreRevision', () => noteRepository.restoreRevision(id, revisionId))
    if (note) upsert(note)
    return note
  }

  async function updateNote(id: string, data: NoteUpdate) {
    const current = getById(id)
    if (current) upsert({ ...current, ...data })
    const saved = await run('errors.saveNote', () => noteRepository.updateNote(id, data))
    if (saved) upsert(saved)
    return saved
  }

  function setTags(id: string, tags: string[]) {
    return updateNote(id, { tags: normalizeTags(tags) })
  }

  async function renameTag(from: string, to: string) {
    await run('errors.renameTag', () => noteRepository.renameTag(from, to))
  }

  async function deleteTag(name: string) {
    const key = tagKey(name)
    // Optimistic: the confirmed notes arrive through `notes:changed`.
    for (const note of notes.value) {
      if (note.tags.some((tag) => tagKey(tag) === key)) {
        upsert({ ...note, tags: note.tags.filter((tag) => tagKey(tag) !== key) })
      }
    }
    await run('errors.deleteTag', () => noteRepository.deleteTag(name))
  }

  /** Manual order of one list, ids in display order. */
  async function reorderNotes(ids: string[]) {
    ids.forEach((id, index) => {
      const note = getById(id)
      if (note) upsert({ ...note, sortOrder: index })
    })
    await run('errors.saveOrder', () => noteRepository.reorderNotes(ids))
  }

  async function emptyTrash() {
    const ids = deletedNotes.value.map((note) => note.id)
    const count = await run('errors.emptyTrash', () => noteRepository.emptyTrash())
    if (count === undefined) return
    ids.forEach(remove)
    notice.value = count > 0 ? t('notices.trashEmptied', { count }) : t('notices.trashAlreadyEmpty')
  }

  // ---- Batch actions (multi-select in the manager) ----

  async function forEachNote(ids: readonly string[], action: (id: string) => Promise<unknown>) {
    for (const id of ids) await action(id)
  }

  const archiveNotes = (ids: readonly string[]) => forEachNote(ids, archiveNote)
  const deleteNotes = (ids: readonly string[]) => forEachNote(ids, deleteNote)
  const deleteNotesPermanently = (ids: readonly string[]) => forEachNote(ids, deleteNotePermanently)
  const restoreNotes = (ids: readonly string[]) =>
    forEachNote(ids, (id) => (getById(id)?.status === 'deleted' ? restoreDeletedNote(id) : restoreNote(id)))
  const setColorOfNotes = (ids: readonly string[], color: NoteColor) =>
    forEachNote(ids, (id) => (getById(id)?.color === color ? Promise.resolve() : updateNote(id, { color })))

  function addTagToNotes(ids: readonly string[], name: string) {
    return forEachNote(ids, async (id) => {
      const note = getById(id)
      if (note && !hasTag(note, name)) await setTags(id, [...note.tags, name])
    })
  }

  function removeTagFromNotes(ids: readonly string[], name: string) {
    const key = tagKey(name)
    return forEachNote(ids, async (id) => {
      const note = getById(id)
      if (note && hasTag(note, name)) await setTags(id, note.tags.filter((tag) => tagKey(tag) !== key))
    })
  }

  async function changeStatus(id: string, label: MessageKey, task: () => Promise<Note>, closeWindow: boolean) {
    const note = await run(label, task)
    if (!note) return undefined
    upsert(note)
    if (closeWindow) await closeNoteWindow(id)
    return note
  }

  const archiveNote = (id: string) => changeStatus(id, 'errors.archive', () => noteRepository.archiveNote(id), true)
  const restoreNote = (id: string) => changeStatus(id, 'errors.restore', () => noteRepository.restoreNote(id), false)
  const deleteNote = (id: string) => changeStatus(id, 'errors.delete', () => noteRepository.softDeleteNote(id), true)
  const restoreDeletedNote = (id: string) =>
    changeStatus(id, 'errors.restore', () => noteRepository.restoreDeletedNote(id), false)

  async function deleteNotePermanently(id: string) {
    await closeNoteWindow(id)
    const done = await run('errors.deletePermanently', async () => {
      await noteRepository.deleteNotePermanently(id)
      return true
    })
    if (done) remove(id)
  }

  async function openNoteWindow(id: string) {
    await run('errors.openNote', () => windowService.openNoteWindow(id))
  }

  async function closeNoteWindow(id: string) {
    await run('errors.closeNote', () => windowService.closeNoteWindow(id))
  }

  async function showAllNotes() {
    const shown = await run('errors.showNotes', () => windowService.showAllNotes())
    if (shown !== undefined) notice.value = shown > 0 ? t('notices.notesShown', { count: shown }) : t('notices.nothingToShow')
  }

  async function arrangeNotes(layout: ArrangeLayout, anchor: ArrangeAnchor) {
    const moved = await run('errors.arrangeNotes', () => windowService.arrangeNoteWindows(layout, anchor))
    if (moved !== undefined) notice.value = moved > 0 ? t('notices.notesArranged', { count: moved }) : t('notices.noNotesOnDesktop')
  }

  /** Desktop workspace: only these notes stay open. */
  async function showOnlyNotes(ids: string[]) {
    const shown = await run('errors.showNotes', () => windowService.showOnlyNotes(ids))
    if (shown !== undefined) notice.value = t('notices.showOnly', { count: shown })
  }

  async function toggleNotesHidden() {
    const hidden = await run('errors.toggleHidden', () => windowService.toggleNotesHidden())
    if (hidden !== undefined) notice.value = hidden ? t('notices.notesHidden') : t('notices.notesUnhidden')
  }

  async function exportNotes() {
    const result = await run('errors.export', () => noteRepository.exportNotes())
    if (result) notice.value = t('notices.exported', { count: result.count, images: result.images, path: result.path })
  }

  async function importNotes() {
    const result = await run('errors.import', () => noteRepository.importNotes())
    if (!result) return
    // Imported notes also arrive via `notes:changed`; reload to be safe for large files.
    await loadNotes()
    notice.value =
      t('notices.imported', { count: result.count, images: result.images }) +
      (result.skipped > 0 ? t('notices.importSkipped', { count: result.skipped }) : '')
  }

  async function cleanupImages() {
    const result = await run('errors.cleanupImages', () => imageService.cleanupUnused())
    if (!result) return
    notice.value =
      result.count > 0
        ? t('notices.imagesCleaned', { count: result.count, size: formatBytes(result.bytes) })
        : t('notices.noImagesToClean')
  }

  let unsubscribe: (() => void) | null = null

  /** Keeps this window's copy in sync with changes made anywhere else. */
  async function startSync() {
    if (unsubscribe) return
    unsubscribe = await noteRepository.subscribe({ onChanged: upsert, onRemoved: remove })
  }

  function stopSync() {
    unsubscribe?.()
    unsubscribe = null
  }

  return {
    notes,
    loaded,
    lastError,
    notice,
    activeNotes,
    archivedNotes,
    deletedNotes,
    allTags,
    getById,
    loadNotes,
    loadNote,
    createNote,
    createNoteWithTitle,
    loadRevisions,
    addRevision,
    restoreRevision,
    updateNote,
    setTags,
    renameTag,
    deleteTag,
    reorderNotes,
    emptyTrash,
    archiveNotes,
    deleteNotes,
    deleteNotesPermanently,
    restoreNotes,
    setColorOfNotes,
    addTagToNotes,
    removeTagFromNotes,
    archiveNote,
    restoreNote,
    deleteNote,
    restoreDeletedNote,
    deleteNotePermanently,
    openNoteWindow,
    closeNoteWindow,
    showAllNotes,
    arrangeNotes,
    showOnlyNotes,
    toggleNotesHidden,
    exportNotes,
    importNotes,
    cleanupImages,
    startSync,
    stopSync,
  }
})
