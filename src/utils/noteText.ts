import type { Note } from '@/models/note'
import { markdownToPlainText } from '@/utils/markdown'

/**
 * Markdown → plain text is the costly part of searching and listing, and it runs
 * for every note on every keystroke. Cached per content string (bounded, so edited
 * versions don't pile up).
 */
const plainTextCache = new Map<string, string>()
const MAX_CACHED = 2000

export function plainTextOf(content: string): string {
  let text = plainTextCache.get(content)
  if (text === undefined) {
    text = markdownToPlainText(content)
    if (plainTextCache.size >= MAX_CACHED) plainTextCache.delete(plainTextCache.keys().next().value!)
    plainTextCache.set(content, text)
  }
  return text
}

/** Non-empty plain-text lines of the content. */
export function noteLines(note: Pick<Note, 'content'>): string[] {
  return plainTextOf(note.content)
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean)
}

/** Untitled notes use their first line as the heading, like Windows Sticky Notes. */
export function noteHeading(note: Pick<Note, 'title' | 'content'>, lines = noteLines(note)): string {
  return note.title.trim() || lines[0] || '空白便籤'
}

/** Content after the heading, joined on one line. */
export function notePreview(note: Pick<Note, 'title' | 'content'>, lines = noteLines(note)): string {
  return (note.title.trim() ? lines : lines.slice(1)).join('  ')
}
