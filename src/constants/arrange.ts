/** Mirrors `ArrangeLayout` / `ArrangeAnchor` in `src-tauri/src/services/window.rs`. */
export type ArrangeLayout = 'grid' | 'cascade'
export type ArrangeAnchor = 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right' | 'center'

export const ARRANGE_LAYOUTS: readonly ArrangeLayout[] = ['grid', 'cascade']
export const ARRANGE_ANCHORS: readonly ArrangeAnchor[] = ['top-left', 'top-right', 'bottom-left', 'bottom-right', 'center']

export function isArrangeLayout(value: unknown): value is ArrangeLayout {
  return typeof value === 'string' && (ARRANGE_LAYOUTS as readonly string[]).includes(value)
}

export function isArrangeAnchor(value: unknown): value is ArrangeAnchor {
  return typeof value === 'string' && (ARRANGE_ANCHORS as readonly string[]).includes(value)
}
