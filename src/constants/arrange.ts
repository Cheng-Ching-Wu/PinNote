/** Mirrors `ArrangeLayout` / `ArrangeAnchor` in `src-tauri/src/services/window.rs`. */
export type ArrangeLayout = 'grid' | 'cascade'
export type ArrangeAnchor = 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right' | 'center'

export const ARRANGE_LAYOUTS: readonly ArrangeLayout[] = ['grid', 'cascade']
export const ARRANGE_ANCHORS: readonly ArrangeAnchor[] = ['top-left', 'top-right', 'bottom-left', 'bottom-right', 'center']

export const ARRANGE_LAYOUT_LABELS: Record<ArrangeLayout, string> = {
  grid: '並排',
  cascade: '疊放',
}

export const ARRANGE_ANCHOR_LABELS: Record<ArrangeAnchor, string> = {
  'top-left': '左上角',
  'top-right': '右上角',
  'bottom-left': '左下角',
  'bottom-right': '右下角',
  center: '畫面中央',
}

export function isArrangeLayout(value: unknown): value is ArrangeLayout {
  return typeof value === 'string' && (ARRANGE_LAYOUTS as readonly string[]).includes(value)
}

export function isArrangeAnchor(value: unknown): value is ArrangeAnchor {
  return typeof value === 'string' && (ARRANGE_ANCHORS as readonly string[]).includes(value)
}
