/**
 * Links between notes are ordinary Markdown links with a private scheme,
 * e.g. `[會議記錄](pinnote://note/<id>)`, so they survive export / import and
 * search sees only their text.
 */
export const NOTE_LINK_SCHEME = 'pinnote'
const NOTE_LINK_PREFIX = `${NOTE_LINK_SCHEME}://note/`

export function noteLinkHref(id: string): string {
  return `${NOTE_LINK_PREFIX}${encodeURIComponent(id)}`
}

/** The linked note id, or null for any other link. */
export function noteIdFromHref(href: string | null | undefined): string | null {
  if (!href?.startsWith(NOTE_LINK_PREFIX)) return null
  const id = decodeURIComponent(href.slice(NOTE_LINK_PREFIX.length)).replace(/\/+$/, '')
  return id || null
}

/** Longest text after `[[` still treated as a search. */
const MAX_LINK_QUERY = 40

/**
 * The search text when the text before the cursor ends with an unfinished `[[…`,
 * otherwise null. `[[` alone gives an empty query (list recent notes).
 */
export function findLinkQuery(textBeforeCursor: string): string | null {
  const match = /\[\[([^[\]\n]*)$/.exec(textBeforeCursor)
  return match && match[1].length <= MAX_LINK_QUERY ? match[1] : null
}

/** A note offered in the `[[` picker. */
export interface LinkTarget {
  id: string
  heading: string
  archived: boolean
}
