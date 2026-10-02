// @vitest-environment happy-dom
import { createApp, defineComponent, ref } from 'vue'
import { describe, expect, it, vi } from 'vitest'

import { useHorizontalOverflow } from '@/composables/useHorizontalOverflow'

/** A scroll container with fixed metrics (happy-dom does no layout). */
function fakeScroller(clientWidth: number, scrollWidth: number) {
  const el = document.createElement('div')
  let scrollLeft = 0
  Object.defineProperties(el, {
    clientWidth: { get: () => clientWidth },
    scrollWidth: { get: () => scrollWidth },
    scrollLeft: { get: () => scrollLeft, set: (v: number) => (scrollLeft = v) },
  })
  return el
}

function mountWith(el: HTMLElement) {
  let state!: ReturnType<typeof useHorizontalOverflow>
  const app = createApp(
    defineComponent({
      setup() {
        state = useHorizontalOverflow(ref(el))
        return () => null
      },
    }),
  )
  app.mount(document.createElement('div'))
  return { state, app }
}

describe('useHorizontalOverflow', () => {
  it('shows no arrows when everything fits', () => {
    const { state, app } = mountWith(fakeScroller(300, 300))
    expect(state.canScrollLeft.value).toBe(false)
    expect(state.canScrollRight.value).toBe(false)
    app.unmount()
  })

  it('tracks which side has hidden buttons while scrolling', () => {
    const el = fakeScroller(120, 360)
    const { state, app } = mountWith(el)
    expect(state.canScrollLeft.value).toBe(false)
    expect(state.canScrollRight.value).toBe(true)

    el.scrollLeft = 100
    el.dispatchEvent(new Event('scroll'))
    expect(state.canScrollLeft.value).toBe(true)
    expect(state.canScrollRight.value).toBe(true)

    el.scrollLeft = 240 // scrolled to the end
    el.dispatchEvent(new Event('scroll'))
    expect(state.canScrollLeft.value).toBe(true)
    expect(state.canScrollRight.value).toBe(false)
    app.unmount()
  })

  it('auto-scrolls while hovering an arrow and stops at the end', () => {
    const frames: FrameRequestCallback[] = []
    vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => frames.push(cb))
    vi.stubGlobal('cancelAnimationFrame', () => {})
    const runFrame = (time: number) => frames.shift()?.(time)

    const el = fakeScroller(100, 200)
    const { state, app } = mountWith(el)

    state.startAutoScroll(1, 1000) // 1000 px/s
    runFrame(0) // first frame assumes ~16ms
    runFrame(50) // +50ms → +50px
    expect(el.scrollLeft).toBeCloseTo(66, 0)

    state.stopAutoScroll()
    expect(frames.length).toBeGreaterThan(0)
    frames.length = 0
    runFrame(100)
    expect(el.scrollLeft).toBeCloseTo(66, 0) // nothing scheduled after stop

    state.startAutoScroll(1, 1000)
    for (let t = 0; t < 2000 && frames.length; t += 50) runFrame(t)
    expect(state.canScrollRight.value).toBe(false)
    expect(frames.length).toBe(0) // loop ended by itself at the end

    vi.unstubAllGlobals()
    app.unmount()
  })

  it('maps the vertical mouse wheel to sideways scrolling', () => {
    const el = fakeScroller(120, 360)
    const { app } = mountWith(el)
    el.dispatchEvent(new WheelEvent('wheel', { deltaY: 50, cancelable: true }))
    expect(el.scrollLeft).toBe(50)
    app.unmount()
  })
})
