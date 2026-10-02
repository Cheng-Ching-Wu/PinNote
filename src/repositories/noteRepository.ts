import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

import type { Note, NoteUpdate, TransferResult } from '@/models/note'

/** Event names emitted by `src-tauri/src/events.rs`. */
const NOTE_CHANGED = 'notes:changed'
const NOTE_REMOVED = 'notes:removed'

/**
 * The only module that talks to the note commands (and therefore SQLite).
 * Components go through the Pinia store, never through this file directly.
 */
export const noteRepository = {
  getNotes(): Promise<Note[]> {
    return invoke('get_notes')
  },

  getNote(id: string): Promise<Note> {
    return invoke('get_note', { id })
  },

  createNote(): Promise<Note> {
    return invoke('create_note')
  },

  updateNote(id: string, data: NoteUpdate): Promise<Note> {
    return invoke('update_note', { id, data })
  },

  archiveNote(id: string): Promise<Note> {
    return invoke('archive_note', { id })
  },

  restoreNote(id: string): Promise<Note> {
    return invoke('restore_note', { id })
  },

  softDeleteNote(id: string): Promise<Note> {
    return invoke('soft_delete_note', { id })
  },

  restoreDeletedNote(id: string): Promise<Note> {
    return invoke('restore_deleted_note', { id })
  },

  deleteNotePermanently(id: string): Promise<void> {
    return invoke('delete_note_permanently', { id })
  },

  /** Opens a save dialog; resolves to null when cancelled. */
  exportNotes(): Promise<TransferResult | null> {
    return invoke('export_notes')
  },

  /** Opens a file picker; notes whose id already exists are skipped. Resolves to null when cancelled. */
  importNotes(): Promise<TransferResult | null> {
    return invoke('import_notes')
  },

  /** Every window receives changes made from any window, the tray or the shortcut. */
  async subscribe(handlers: {
    onChanged: (note: Note) => void
    onRemoved: (id: string) => void
  }): Promise<UnlistenFn> {
    const unlistenChanged = await listen<Note>(NOTE_CHANGED, (event) => handlers.onChanged(event.payload))
    const unlistenRemoved = await listen<string>(NOTE_REMOVED, (event) => handlers.onRemoved(event.payload))
    return () => {
      unlistenChanged()
      unlistenRemoved()
    }
  },
}
