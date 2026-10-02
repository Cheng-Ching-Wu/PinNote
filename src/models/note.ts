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

  createdAt: string
  updatedAt: string
  deletedAt: string | null
}

/** Fields that can be changed through `updateNote`. Status and open state have dedicated actions. */
export type NoteUpdate = Partial<
  Pick<Note, 'title' | 'content' | 'color' | 'width' | 'height' | 'alwaysOnTop' | 'collapsed'> & {
    x: number
    y: number
  }
>

export interface TransferResult {
  path: string
  /** Notes written (export) or added (import). */
  count: number
  /** Import only: notes skipped because they already exist. */
  skipped: number
}

export interface WindowGeometry {
  x: number
  y: number
  width: number
  height: number
}
