import type { NoteColor } from '@/models/note'

export const NOTE_COLORS: readonly NoteColor[] = ['yellow', 'blue', 'green', 'pink', 'gray']

export const DEFAULT_NOTE_COLOR: NoteColor = 'yellow'

/**
 * Each class defines the `--note-*` CSS variables (see `src/styles/note-colors.css`).
 * Components only reference the variables, e.g. `bg-(--note-body)`, never a concrete color.
 */
export const NOTE_COLOR_CLASSES: Record<NoteColor, string> = {
  yellow: 'note-yellow',
  blue: 'note-blue',
  green: 'note-green',
  pink: 'note-pink',
  gray: 'note-gray',
}
