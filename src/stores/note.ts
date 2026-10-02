import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import type { Note, NoteUpdate } from '@/models/note'
import { noteRepository } from '@/repositories/noteRepository'
import { windowService } from '@/services/windowService'
import { markdownToPlainText } from '@/utils/markdown'

const byUpdatedDesc = (a: Note, b: Note) => b.updatedAt.localeCompare(a.updatedAt)
const byDeletedDesc = (a: Note, b: Note) => (b.deletedAt ?? '').localeCompare(a.deletedAt ?? '')

export function matchesQuery(note: Note, query: string): boolean {
  const q = query.trim().toLowerCase()
  if (!q) return true
  return note.title.toLowerCase().includes(q) || markdownToPlainText(note.content).toLowerCase().includes(q)
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

  async function updateNote(id: string, data: NoteUpdate) {
    const current = getById(id)
    if (current) upsert({ ...current, ...data })
    const saved = await run('儲存便籤失敗', () => noteRepository.updateNote(id, data))
    if (saved) upsert(saved)
    return saved
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

  async function toggleNotesHidden() {
    const hidden = await run('隱藏 / 顯示便籤失敗', () => windowService.toggleNotesHidden())
    if (hidden !== undefined) notice.value = hidden ? '已暫時隱藏所有便籤（Ctrl+Alt+H 可再顯示）' : '已重新顯示便籤'
  }

  async function exportNotes() {
    const result = await run('匯出失敗', () => noteRepository.exportNotes())
    if (result) notice.value = `已匯出 ${result.count} 張便籤到 ${result.path}`
  }

  async function importNotes() {
    const result = await run('匯入失敗', () => noteRepository.importNotes())
    if (!result) return
    // Imported notes also arrive via `notes:changed`; reload to be safe for large files.
    await loadNotes()
    notice.value =
      `已匯入 ${result.count} 張便籤` + (result.skipped > 0 ? `，略過 ${result.skipped} 張已存在的便籤` : '')
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
    getById,
    loadNotes,
    loadNote,
    createNote,
    updateNote,
    archiveNote,
    restoreNote,
    deleteNote,
    restoreDeletedNote,
    deleteNotePermanently,
    openNoteWindow,
    closeNoteWindow,
    showAllNotes,
    toggleNotesHidden,
    exportNotes,
    importNotes,
    startSync,
    stopSync,
  }
})
