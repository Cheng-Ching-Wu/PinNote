import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import type { ArrangeAnchor, ArrangeLayout } from '@/constants/arrange'
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
  async function run<T>(label: string, task: () => Promise<T>): Promise<T | undefined> {
    try {
      return await task()
    } catch (error) {
      lastError.value = `${label}: ${String(error)}`
      console.error(`[note-store] ${label}`, error)
      return undefined
    }
  }

  async function loadNotes() {
    const list = await run('載入便籤失敗', () => noteRepository.getNotes())
    if (list) {
      notes.value = list
      loaded.value = true
    }
  }

  async function loadNote(id: string) {
    const note = await run('載入便籤失敗', () => noteRepository.getNote(id))
    if (note) upsert(note)
    return note
  }

  /** Creates a note, opens its window and lets it take focus. */
  async function createNote() {
    const note = await run('新增便籤失敗', () => noteRepository.createNote())
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
    const created = await run('新增便籤失敗', () => noteRepository.createNote())
    if (!created) return undefined
    const note = (await run('新增便籤失敗', () => noteRepository.updateNote(created.id, { title }))) ?? created
    upsert(note)
    if (open) await openNoteWindow(note.id)
    else await closeNoteWindow(note.id) // records `isOpen = false` (no window exists)
    return note
  }

  function addRevision(id: string, title: string, content: string) {
    return run('保留版本失敗', () => noteRepository.addRevision(id, title, content))
  }

  function loadRevisions(id: string) {
    return run('載入歷史版本失敗', () => noteRepository.getRevisions(id))
  }

  async function restoreRevision(id: string, revisionId: number) {
    const note = await run('還原版本失敗', () => noteRepository.restoreRevision(id, revisionId))
    if (note) upsert(note)
    return note
  }

  async function updateNote(id: string, data: NoteUpdate) {
    const current = getById(id)
    if (current) upsert({ ...current, ...data })
    const saved = await run('儲存便籤失敗', () => noteRepository.updateNote(id, data))
    if (saved) upsert(saved)
    return saved
  }

  function setTags(id: string, tags: string[]) {
    return updateNote(id, { tags: normalizeTags(tags) })
  }

  async function renameTag(from: string, to: string) {
    await run('重新命名標籤失敗', () => noteRepository.renameTag(from, to))
  }

  async function deleteTag(name: string) {
    const key = tagKey(name)
    // Optimistic: the confirmed notes arrive through `notes:changed`.
    for (const note of notes.value) {
      if (note.tags.some((tag) => tagKey(tag) === key)) {
        upsert({ ...note, tags: note.tags.filter((tag) => tagKey(tag) !== key) })
      }
    }
    await run('刪除標籤失敗', () => noteRepository.deleteTag(name))
  }

  /** Manual order of one list, ids in display order. */
  async function reorderNotes(ids: string[]) {
    ids.forEach((id, index) => {
      const note = getById(id)
      if (note) upsert({ ...note, sortOrder: index })
    })
    await run('儲存排序失敗', () => noteRepository.reorderNotes(ids))
  }

  async function emptyTrash() {
    const ids = deletedNotes.value.map((note) => note.id)
    const count = await run('清空垃圾桶失敗', () => noteRepository.emptyTrash())
    if (count === undefined) return
    ids.forEach(remove)
    notice.value = count > 0 ? `已永久刪除 ${count} 張便籤` : '垃圾桶是空的'
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

  async function changeStatus(id: string, label: string, task: () => Promise<Note>, closeWindow: boolean) {
    const note = await run(label, task)
    if (!note) return undefined
    upsert(note)
    if (closeWindow) await closeNoteWindow(id)
    return note
  }

  const archiveNote = (id: string) => changeStatus(id, '封存失敗', () => noteRepository.archiveNote(id), true)
  const restoreNote = (id: string) => changeStatus(id, '還原失敗', () => noteRepository.restoreNote(id), false)
  const deleteNote = (id: string) => changeStatus(id, '刪除失敗', () => noteRepository.softDeleteNote(id), true)
  const restoreDeletedNote = (id: string) =>
    changeStatus(id, '還原失敗', () => noteRepository.restoreDeletedNote(id), false)

  async function deleteNotePermanently(id: string) {
    await closeNoteWindow(id)
    const done = await run('永久刪除失敗', async () => {
      await noteRepository.deleteNotePermanently(id)
      return true
    })
    if (done) remove(id)
  }

  async function openNoteWindow(id: string) {
    await run('開啟便籤失敗', () => windowService.openNoteWindow(id))
  }

  async function closeNoteWindow(id: string) {
    await run('關閉便籤失敗', () => windowService.closeNoteWindow(id))
  }

  async function showAllNotes() {
    const shown = await run('顯示便籤失敗', () => windowService.showAllNotes())
    if (shown !== undefined) notice.value = shown > 0 ? `已顯示 ${shown} 張便籤` : '沒有可顯示的便籤'
  }

  async function arrangeNotes(layout: ArrangeLayout, anchor: ArrangeAnchor) {
    const moved = await run('整理便籤失敗', () => windowService.arrangeNoteWindows(layout, anchor))
    if (moved !== undefined) notice.value = moved > 0 ? `已整理 ${moved} 張便籤` : '桌面上沒有便籤'
  }

  /** Desktop workspace: only these notes stay open. */
  async function showOnlyNotes(ids: string[]) {
    const shown = await run('顯示便籤失敗', () => windowService.showOnlyNotes(ids))
    if (shown !== undefined) notice.value = `桌面只顯示 ${shown} 張便籤（其他已關閉，可用「顯示全部便籤」叫回）`
  }

  async function toggleNotesHidden() {
    const hidden = await run('隱藏 / 顯示便籤失敗', () => windowService.toggleNotesHidden())
    if (hidden !== undefined) notice.value = hidden ? '已暫時隱藏所有便籤（Ctrl+Alt+H 可再顯示）' : '已重新顯示便籤'
  }

  async function exportNotes() {
    const result = await run('匯出失敗', () => noteRepository.exportNotes())
    if (result) notice.value = `已匯出 ${result.count} 張便籤、${result.images} 張圖片到 ${result.path}`
  }

  async function importNotes() {
    const result = await run('匯入失敗', () => noteRepository.importNotes())
    if (!result) return
    // Imported notes also arrive via `notes:changed`; reload to be safe for large files.
    await loadNotes()
    notice.value =
      `已匯入 ${result.count} 張便籤、${result.images} 張圖片` +
      (result.skipped > 0 ? `，略過 ${result.skipped} 張已存在的便籤` : '')
  }

  async function cleanupImages() {
    const result = await run('清理圖片失敗', () => imageService.cleanupUnused())
    if (!result) return
    notice.value =
      result.count > 0
        ? `已刪除 ${result.count} 張未使用的圖片，釋放 ${formatBytes(result.bytes)}`
        : '沒有可清理的圖片（24 小時內新增的圖片會保留）'
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
