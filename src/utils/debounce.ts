export interface Debounced<A extends unknown[]> {
  (...args: A): void
  /** Drops the pending call. */
  cancel(): void
  readonly pending: boolean
}

export function debounce<A extends unknown[]>(fn: (...args: A) => void, wait: number): Debounced<A> {
  let timer: ReturnType<typeof setTimeout> | undefined

  const debounced = ((...args: A) => {
    clearTimeout(timer)
    timer = setTimeout(() => {
      timer = undefined
      fn(...args)
    }, wait)
  }) as Debounced<A>

  debounced.cancel = () => {
    clearTimeout(timer)
    timer = undefined
  }
  Object.defineProperty(debounced, 'pending', { get: () => timer !== undefined })

  return debounced
}
