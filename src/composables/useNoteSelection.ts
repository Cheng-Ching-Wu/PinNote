import { computed, ref, type Ref, watch } from 'vue'

/**
 * Multi-select over a list of ids in display order: Ctrl+click toggles,
 * Shift+click selects a range from the last clicked item. Ids that leave the
 * list (archived, filtered out…) drop out of the selection.
 */
export function useNoteSelection(visibleIds: Readonly<Ref<readonly string[]>>) {
  const selected = ref<ReadonlySet<string>>(new Set())
  let anchor: string | null = null

  const count = computed(() => selected.value.size)
  /** Selection mode: on while anything is selected. */
  const selecting = computed(() => selected.value.size > 0)
  /** Selected ids in display order. */
  const ids = computed(() => visibleIds.value.filter((id) => selected.value.has(id)))
  const allSelected = computed(() => visibleIds.value.length > 0 && count.value === visibleIds.value.length)

  watch(visibleIds, (list) => {
    const present = new Set(list)
    if ([...selected.value].some((id) => !present.has(id))) {
      selected.value = new Set([...selected.value].filter((id) => present.has(id)))
    }
    if (anchor && !present.has(anchor)) anchor = null
  })

  function isSelected(id: string) {
    return selected.value.has(id)
  }

  function toggle(id: string) {
    const next = new Set(selected.value)
    if (next.has(id)) next.delete(id)
    else next.add(id)
    selected.value = next
    anchor = id
  }

  /** Adds everything between the anchor and `id` (inclusive). */
  function selectRange(id: string) {
    const list = visibleIds.value
    const from = anchor ? list.indexOf(anchor) : -1
    const to = list.indexOf(id)
    if (from === -1 || to === -1) return toggle(id)
    const [start, end] = from < to ? [from, to] : [to, from]
    selected.value = new Set([...selected.value, ...list.slice(start, end + 1)])
  }

  function selectAll() {
    selected.value = new Set(visibleIds.value)
  }

  function clear() {
    selected.value = new Set()
    anchor = null
  }

  return { count, selecting, ids, allSelected, isSelected, toggle, selectRange, selectAll, clear }
}
