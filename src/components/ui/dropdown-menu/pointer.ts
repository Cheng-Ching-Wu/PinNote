/** Last known pointer position, shared by the menus. Starts off-screen so nothing matches before the first move. */
export const lastPointer = { x: -1, y: -1 }

if (typeof window !== 'undefined') {
  window.addEventListener(
    'pointermove',
    (event) => {
      lastPointer.x = event.clientX
      lastPointer.y = event.clientY
    },
    { capture: true, passive: true },
  )
}

/** Whether the pointer is currently over `element` (and not over something stacked on top of it). */
export function isPointerOver(element: Element | null | undefined): boolean {
  if (!element) return false
  const under = document.elementFromPoint(lastPointer.x, lastPointer.y)
  return !!under && element.contains(under)
}
