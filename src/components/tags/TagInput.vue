<script setup lang="ts">
import { computed, type HTMLAttributes, ref, useTemplateRef, watch } from 'vue'
import { Plus } from 'lucide-vue-next'

import { t } from '@/i18n'
import { cn } from '@/lib/utils'
import { MAX_TAG_LENGTH, normalizeTag, tagKey } from '@/utils/tags'

const props = withDefaults(
  defineProps<{
    /** Tags already on the note — left out of the suggestions. */
    current: readonly string[]
    /** Every known tag name. */
    suggestions: readonly string[]
    /** Where the suggestion list opens; `top` for inputs at the bottom of a note. */
    placement?: 'top' | 'bottom'
    placeholder?: string
    class?: HTMLAttributes['class']
    /** Tone of the suggestion list. */
    listClass?: HTMLAttributes['class']
    tagColor?: (name: string) => string
  }>(),
  { placement: 'bottom', placeholder: () => t('tags.inputPlaceholder') },
)

const emit = defineEmits<{
  add: [name: string]
  /** Backspace in an empty input. */
  'remove-last': []
  /** Escape, or focus left the input. */
  done: []
}>()

const text = ref('')
const highlighted = ref(-1)
const inputRef = useTemplateRef<HTMLInputElement>('inputRef')

const MAX_SUGGESTIONS = 6

const matches = computed(() => {
  const used = new Set(props.current.map(tagKey))
  const query = tagKey(text.value.trim().replace(/^#+/, ''))
  return props.suggestions
    .filter((name) => !used.has(tagKey(name)) && tagKey(name).includes(query))
    // Prefix matches first; `sort` is stable, so the alphabetical order is kept otherwise.
    .sort((a, b) => Number(!tagKey(a).startsWith(query)) - Number(!tagKey(b).startsWith(query)))
    .slice(0, MAX_SUGGESTIONS)
})

/** The typed name, when it isn't already an existing tag (then "新增" is offered). */
const newName = computed(() => {
  const name = normalizeTag(text.value)
  if (!name) return null
  const key = tagKey(name)
  if (props.current.some((tag) => tagKey(tag) === key)) return null
  return props.suggestions.some((tag) => tagKey(tag) === key) ? null : name
})

const open = computed(() => matches.value.length > 0 || !!newName.value)

watch(text, () => (highlighted.value = -1))

function add(name: string) {
  const tag = normalizeTag(name)
  text.value = ''
  if (tag) emit('add', tag)
}

function commit() {
  if (highlighted.value >= 0 && highlighted.value < matches.value.length) add(matches.value[highlighted.value])
  else if (text.value.trim()) add(text.value)
}

function move(step: 1 | -1) {
  const count = matches.value.length
  if (count === 0) return
  // -1 is the input itself; wrap around through it.
  const next = highlighted.value + step
  highlighted.value = next >= count ? -1 : next < -1 ? count - 1 : next
}

function onKeydown(event: KeyboardEvent) {
  if (event.isComposing) return
  switch (event.key) {
    case 'Enter':
    case ',':
    case '，':
      event.preventDefault()
      commit()
      break
    case 'ArrowDown':
      event.preventDefault()
      move(1)
      break
    case 'ArrowUp':
      event.preventDefault()
      move(-1)
      break
    case 'Backspace':
      if (!text.value) emit('remove-last')
      break
    case 'Escape':
      // First Escape clears the text; with nothing typed it may also close a surrounding dialog.
      if (text.value) {
        event.preventDefault()
        event.stopPropagation()
      }
      text.value = ''
      emit('done')
      break
  }
}

function onBlur() {
  // Typed but not confirmed: keep it rather than silently dropping it.
  if (text.value.trim()) add(text.value)
  emit('done')
}

defineExpose({ focus: () => inputRef.value?.focus() })
</script>

<template>
  <div class="relative min-w-0">
    <input
      ref="inputRef"
      v-model="text"
      :placeholder="placeholder"
      :maxlength="MAX_TAG_LENGTH + 1"
      spellcheck="false"
      :class="cn('h-5 w-full min-w-16 bg-transparent text-[11px] outline-none', props.class)"
      @keydown="onKeydown"
      @blur="onBlur"
    />
    <ul
      v-if="open"
      :class="
        cn(
          'absolute left-0 z-20 max-h-44 w-44 overflow-y-auto rounded-md border bg-card p-0.5 text-[12px] text-foreground shadow-md',
          placement === 'top' ? 'bottom-full mb-1' : 'top-full mt-1',
          listClass,
        )
      "
    >
      <!-- mousedown.prevent keeps focus in the input, so blur doesn't fire first. -->
      <li
        v-for="(name, index) in matches"
        :key="name"
        :class="
          cn(
            'flex h-6 cursor-pointer items-center gap-1.5 rounded-sm px-1.5',
            index === highlighted ? 'bg-accent' : 'hover:bg-accent',
          )
        "
        @mousedown.prevent="add(name)"
      >
        <span
          v-if="tagColor"
          class="size-1.5 shrink-0 rounded-full"
          :style="{ backgroundColor: tagColor(name) }"
        />
        <span class="truncate">{{ name }}</span>
      </li>
      <li
        v-if="newName"
        class="flex h-6 cursor-pointer items-center gap-1.5 rounded-sm px-1.5 text-muted-foreground hover:bg-accent"
        @mousedown.prevent="add(newName)"
      >
        <Plus class="size-3 shrink-0" />
        <span class="truncate">{{ t('tags.addNamed', { name: newName }) }}</span>
      </li>
    </ul>
  </div>
</template>
