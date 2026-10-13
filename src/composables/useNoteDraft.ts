import { computed, ref } from 'vue'

import type { Note, NoteUpdate } from '@/models/note'
import { useNoteStore } from '@/stores/note'
import { debounce } from '@/utils/debounce'

/**
 * Local editable copy of a note's title / content with debounced auto-save.
 *
 * The draft is the source of truth while the window is open, so echoes of our
 * own (older) saves coming back through the store never overwrite newer typing.
 */
export function useNoteDraft(id: string, delay: number) {
  const store = useNoteStore()

  const titleDraft = ref('')
  const contentDraft = ref('')
  let pending: NoteUpdate = {}

  const scheduleSave = debounce(() => void flush(), delay)

  function init(note: Note) {
    titleDraft.value = note.title
    contentDraft.value = note.content
  }

  const title = computed({
    get: () => titleDraft.value,
    set: (value: string) => {
      titleDraft.value = value
      pending.title = value
      scheduleSave()
    },
  })

  const content = computed({
    get: () => contentDraft.value,
    set: (value: string) => {
      contentDraft.value = value
      pending.content = value
      scheduleSave()
    },
  })

  /** Saves immediately whatever is still waiting for the debounce. */
  async function flush() {
    scheduleSave.cancel()
    if (Object.keys(pending).length === 0) return
    const patch = pending
    pending = {}
    await store.updateNote(id, patch)
  }

  /** Typing that hasn't been saved yet. */
  const hasPending = () => Object.keys(pending).length > 0

  return { title, content, init, flush, hasPending }
}
