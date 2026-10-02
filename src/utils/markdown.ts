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
      // links / images → their text
      .replace(/!?\[([^\]]*)\]\([^)]*\)/g, '$1')
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
