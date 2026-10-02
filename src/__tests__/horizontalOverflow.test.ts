// @vitest-environment happy-dom
import { createApp, defineComponent, ref } from 'vue'
import { describe, expect, it } from 'vitest'

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

  it('maps the vertical mouse wheel to sideways scrolling', () => {
    const el = fakeScroller(120, 360)
    const { app } = mountWith(el)
    el.dispatchEvent(new WheelEvent('wheel', { deltaY: 50, cancelable: true }))
    expect(el.scrollLeft).toBe(50)
    app.unmount()
  })
})
