import type { Note } from '@/models/note'
import { matchesQuery } from '@/stores/note'
import { noteHeading, noteLines, notePreview } from '@/utils/noteText'
import { parseTagQuery } from '@/utils/tags'

export interface SearchResult {
  note: Note
  heading: string
  /** The line that matched, or the start of the note. */
  snippet: string
}

/**
 * Quick search over active and archived notes: heading matches first, then
 * content matches, active before archived, most recently modified first.
 * An empty query lists the most recent active notes.
 */
export function searchNotes(notes: readonly Note[], query: string, limit = 8): SearchResult[] {
  const { tags, text } = parseTagQuery(query)
  const term = text.toLowerCase()
  const searchable = notes.filter((note) =>
    tags.length === 0 && !term ? note.status === 'active' : note.status !== 'deleted',
  )

  return searchable
    .filter((note) => matchesQuery(note, query))
    .map((note) => {
      const lines = noteLines(note)
      const heading = noteHeading(note, lines)
      const headingHit = !!term && heading.toLowerCase().includes(term)
      const line = term && !headingHit ? lines.find((l) => l.toLowerCase().includes(term)) : undefined
      return { note, heading, snippet: line ?? notePreview(note, lines), rank: headingHit || !term ? 0 : 1 }
    })
    .sort(
      (a, b) =>
        a.rank - b.rank ||
        Number(a.note.status !== 'active') - Number(b.note.status !== 'active') ||
        b.note.updatedAt.localeCompare(a.note.updatedAt),
    )
    .slice(0, limit)
    .map(({ note, heading, snippet }) => ({ note, heading, snippet }))
}
