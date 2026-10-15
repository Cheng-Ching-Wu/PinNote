import { collatorLocale } from '@/i18n'
import type { Note } from '@/models/note'

/** Longest tag name, in characters. Mirrors `MAX_TAG_LENGTH` in `src-tauri/src/models/note.rs`. */
export const MAX_TAG_LENGTH = 30

/**
 * Cleans a single tag name: no leading `#`, single spaces, at most `MAX_TAG_LENGTH`
 * characters. Returns null when nothing is left. Mirrors `normalize_tag` in Rust.
 */
export function normalizeTag(raw: string): string | null {
  const name = Array.from(raw.trim().replace(/^#+/, '').split(/\s+/).filter(Boolean).join(' '))
    .slice(0, MAX_TAG_LENGTH)
    .join('')
    .trimEnd()
  return name || null
}

/** Tags are compared without case: "Work" and "work" are the same tag. */
export function tagKey(name: string): string {
  return name.toLowerCase()
}

/** Normalises every tag and drops duplicates, keeping the first spelling and the order. */
export function normalizeTags(tags: readonly string[]): string[] {
  const seen = new Set<string>()
  const result: string[] = []
  for (const raw of tags) {
    const tag = normalizeTag(raw)
    if (!tag || seen.has(tagKey(tag))) continue
    seen.add(tagKey(tag))
    result.push(tag)
  }
  return result
}

export function hasTag(note: Pick<Note, 'tags'>, name: string): boolean {
  const key = tagKey(name)
  return note.tags.some((tag) => tagKey(tag) === key)
}

export interface TagSummary {
  name: string
  key: string
  count: number
}

/** Every tag used by the given notes with how many notes use it, sorted by name. */
export function collectTags(notes: readonly Pick<Note, 'tags'>[]): TagSummary[] {
  const byKey = new Map<string, TagSummary>()
  for (const note of notes) {
    for (const tag of note.tags) {
      const key = tagKey(tag)
      const summary = byKey.get(key)
      if (summary) summary.count++
      else byKey.set(key, { name: tag, key, count: 1 })
    }
  }
  return [...byKey.values()].sort((a, b) => a.name.localeCompare(b.name, collatorLocale()))
}

/** Pseudo tag key for the "未分類" filter. Real keys can't be empty. */
export const UNTAGGED_KEY = ''

/**
 * OR filter: a note matches when it has any of the selected tags
 * (or has no tags while `UNTAGGED_KEY` is selected). No selection matches everything.
 */
export function matchesTagFilter(note: Pick<Note, 'tags'>, selected: ReadonlySet<string>): boolean {
  if (selected.size === 0) return true
  if (note.tags.length === 0) return selected.has(UNTAGGED_KEY)
  return note.tags.some((tag) => selected.has(tagKey(tag)))
}

/**
 * Splits a search query into `#tag` terms and the remaining text, e.g.
 * `#工作 會議` → tags ["工作"], text "會議".
 */
export function parseTagQuery(query: string): { tags: string[]; text: string } {
  const tags: string[] = []
  const words: string[] = []
  for (const word of query.trim().split(/\s+/).filter(Boolean)) {
    const tag = word.startsWith('#') ? normalizeTag(word) : null
    if (tag) tags.push(tag)
    else words.push(word)
  }
  return { tags, text: words.join(' ') }
}
