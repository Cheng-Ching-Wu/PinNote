/**
 * Strips Markdown syntax for list previews and search, keeping the line structure.
 * Intentionally lightweight — it only needs to cover what the note editor produces.
 */
export function markdownToPlainText(markdown: string): string {
  return (
    markdown
      .replace(/&nbsp;/g, ' ')
      // code fences
      .replace(/^\s*```.*$/gm, '')
      // block prefixes: headings, quotes, task items, list bullets / numbers
      .replace(/^\s{0,3}#{1,6}\s+/gm, '')
      .replace(/^\s*(?:>\s?)+/gm, '')
      .replace(/^\s*[-*+]\s+\[[ xX]\]\s+/gm, '')
      .replace(/^\s*(?:[-*+]|\d+[.)])\s+/gm, '')
      // horizontal rules
      .replace(/^\s*(?:[-*_]\s*){3,}$/gm, '')
      // images carry no text; links → their text
      .replace(/!\[[^\]]*\]\([^)]*\)/g, '')
      .replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
      // inline marks
      .replace(/(\*\*|__|~~|==|\+\+)(.+?)\1/g, '$2')
      .replace(/(?<![\w*])\*(?!\s)(.+?)\*(?!\w)/g, '$1')
      .replace(/(?<!\w)_(?!\s)(.+?)_(?!\w)/g, '$1')
      .replace(/`([^`]+)`/g, '$1')
      // backslash escapes
      .replace(/\\([\\`*_{}[\]()#+\-.!=~|>])/g, '$1')
      // hard-break trailing spaces
      .replace(/[ \t]+$/gm, '')
  )
}

const NAMED_ENTITIES: Record<string, string> = { amp: '&', lt: '<', gt: '>', quot: '"', apos: "'", nbsp: ' ' }

function decodeEntities(text: string): string {
  return text.replace(/&(#x[\da-f]+|#\d+|[a-z]+);/gi, (match, entity: string) => {
    if (entity[0] === '#') {
      const code = entity[1].toLowerCase() === 'x' ? parseInt(entity.slice(2), 16) : parseInt(entity.slice(1), 10)
      return Number.isFinite(code) ? String.fromCodePoint(code) : match
    }
    return NAMED_ENTITIES[entity.toLowerCase()] ?? match
  })
}

/**
 * Letters and digits only — a formatting-independent fingerprint of the text.
 * Comparing the stored Markdown with what the editor loaded reveals content the
 * editor could not represent (it would be silently dropped on the next save).
 */
export function contentFingerprint(text: string): string {
  return decodeEntities(text).normalize('NFKC').replace(/[^\p{L}\p{N}]/gu, '')
}

/** True when loading `markdown` into the editor lost text (editor text given as `loadedText`). */
export function isLossyLoad(markdown: string, loadedText: string): boolean {
  return contentFingerprint(markdownToPlainText(markdown)) !== contentFingerprint(loadedText)
}
