import { tagKey } from '@/utils/tags'

export type TagColor = 'red' | 'orange' | 'yellow' | 'green' | 'teal' | 'blue' | 'violet' | 'pink' | 'gray'

export const TAG_COLORS: readonly TagColor[] = ['red', 'orange', 'yellow', 'green', 'teal', 'blue', 'violet', 'pink', 'gray']

/** Dot colors — saturated enough to read on every note color and on the manager's card background. */
export const TAG_COLOR_VALUES: Record<TagColor, string> = {
  red: '#e5484d',
  orange: '#f76b15',
  yellow: '#e2a400',
  green: '#30a46c',
  teal: '#12a594',
  blue: '#0090ff',
  violet: '#8e4ec6',
  pink: '#d6409f',
  gray: '#8b8d98',
}

export function isTagColor(value: string | undefined): value is TagColor {
  return !!value && (TAG_COLORS as readonly string[]).includes(value)
}

/** Stable color for tags the user hasn't colored, so different tags still look different. */
export function autoTagColor(name: string): TagColor {
  let hash = 0
  for (const char of tagKey(name)) hash = (hash * 31 + char.codePointAt(0)!) >>> 0
  const automatic = TAG_COLORS.filter((color) => color !== 'gray')
  return automatic[hash % automatic.length]
}

/** The chosen color from settings, or the automatic one. */
export function resolveTagColor(name: string, colors: Record<string, string>): TagColor {
  const chosen = colors[tagKey(name)]
  return isTagColor(chosen) ? chosen : autoTagColor(name)
}
