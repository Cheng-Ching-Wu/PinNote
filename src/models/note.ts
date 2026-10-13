export type NoteStatus =
  | 'active'
  | 'archived'
  | 'deleted'

export type NoteColor =
  | 'yellow'
  | 'blue'
  | 'green'
  | 'pink'
  | 'gray'

export interface Note {
  id: string

  title: string
  content: string

  color: NoteColor

  /** Normalised tag names (see `utils/tags.ts`), in the order they were added. */
  tags: string[]

  /** Physical (device pixel) outer position of the note window. */
  x: number | null
  y: number | null

  /** Logical size of the window when expanded — never overwritten while collapsed. */
  width: number
  height: number

  alwaysOnTop: boolean
  collapsed: boolean

  /** Whether the note window was open; closed notes are not reopened on launch. */
  isOpen: boolean

  status: NoteStatus

  /** Manual position in the manager list (smaller first). Changed through `reorderNotes`. */
  sortOrder: number

  createdAt: string
  updatedAt: string
  deletedAt: string | null
  /** Last change of any synced field (text, color, tags, status, order). */
  modifiedAt: string
}

/** Fields that can be changed through `updateNote`. Status and open state have dedicated actions. */
export type NoteUpdate = Partial<
  Pick<Note, 'title' | 'content' | 'color' | 'tags' | 'width' | 'height' | 'alwaysOnTop' | 'collapsed'> & {
    x: number
    y: number
  }
>

/** An earlier version of a note's title and content (newest first from the repository). */
export interface NoteRevision {
  id: number
  noteId: string
  title: string
  content: string
  createdAt: string
}

export interface TransferResult {
  path: string
  /** Notes written (export) or added (import). */
  count: number
  /** Import only: notes skipped because they already exist. */
  skipped: number
  /** Images written (export) or added (import). */
  images: number
}

export interface WindowGeometry {
  x: number
  y: number
  width: number
  height: number
}
