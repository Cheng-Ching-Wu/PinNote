import { onBeforeUnmount, onMounted, ref, type Ref } from 'vue'

/**
 * Tracks whether a horizontally scrollable element has hidden content on
 * either side, e.g. a toolbar in a note that was resized narrower.
 */
export function useHorizontalOverflow(element: Readonly<Ref<HTMLElement | null>>) {
  const canScrollLeft = ref(false)
  const canScrollRight = ref(false)

  function update() {
    const el = element.value
    if (!el) return
    // 1px tolerance: scroll positions can be fractional on scaled displays.
    canScrollLeft.value = el.scrollLeft > 1
    canScrollRight.value = el.scrollLeft + el.clientWidth < el.scrollWidth - 1
  }

  /** Scrolls by most of the visible width, keeping a little context. */
  function scrollByPage(direction: -1 | 1) {
    const el = element.value
    if (!el) return
    el.scrollBy({ left: direction * Math.max(el.clientWidth * 0.7, 48), behavior: 'smooth' })
  }

  let frame: number | undefined
  let lastTime: number | undefined

  /**
   * Scrolls continuously (px per second) until `stopAutoScroll` or an end is reached —
   * used while the pointer rests on an arrow.
   */
  function startAutoScroll(direction: -1 | 1, speed = 240) {
    stopAutoScroll()
    const step = (time: number) => {
      const el = element.value
      if (!el) return stopAutoScroll()
      const elapsed = lastTime === undefined ? 16 : Math.min(time - lastTime, 50)
      lastTime = time
      el.scrollLeft += direction * speed * (elapsed / 1000)
      update()
      const atEnd = direction < 0 ? !canScrollLeft.value : !canScrollRight.value
      if (atEnd) return stopAutoScroll()
      frame = requestAnimationFrame(step)
    }
    frame = requestAnimationFrame(step)
  }

  function stopAutoScroll() {
    if (frame !== undefined) cancelAnimationFrame(frame)
    frame = undefined
    lastTime = undefined
  }

  /** Lets a normal (vertical) mouse wheel scroll the bar sideways. */
  function onWheel(event: WheelEvent) {
    const el = element.value
    if (!el || el.scrollWidth <= el.clientWidth || Math.abs(event.deltaX) > Math.abs(event.deltaY)) return
    event.preventDefault()
    el.scrollLeft += event.deltaY
  }

  let resizeObserver: ResizeObserver | undefined

  onMounted(() => {
    const el = element.value
    if (!el) return
    el.addEventListener('scroll', update, { passive: true })
    el.addEventListener('wheel', onWheel, { passive: false })
    // Fires when the note window is resized and when buttons are added / removed.
    resizeObserver = new ResizeObserver(update)
    resizeObserver.observe(el)
    for (const child of Array.from(el.children)) resizeObserver.observe(child)
    update()
  })

  onBeforeUnmount(() => {
    stopAutoScroll()
    const el = element.value
    el?.removeEventListener('scroll', update)
    el?.removeEventListener('wheel', onWheel)
    resizeObserver?.disconnect()
  })

  return { canScrollLeft, canScrollRight, scrollByPage, startAutoScroll, stopAutoScroll, update }
}
