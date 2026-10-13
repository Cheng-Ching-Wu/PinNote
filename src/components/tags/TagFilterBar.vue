<script setup lang="ts">
import { nextTick, useTemplateRef, watch } from 'vue'

import { useHorizontalOverflow } from '@/composables/useHorizontalOverflow'
import { useTagColors } from '@/composables/useTagColors'
import { cn } from '@/lib/utils'
import { type TagSummary, UNTAGGED_KEY } from '@/utils/tags'

const props = defineProps<{
  tags: readonly TagSummary[]
  /** Notes without tags in the current tab. */
  untaggedCount: number
}>()

/** Selected tag keys (`UNTAGGED_KEY` for "未分類"). Notes matching any of them are shown. */
const selected = defineModel<string[]>({ required: true })

const { tagColorValue } = useTagColors()
const scroller = useTemplateRef<HTMLElement>('scroller')
const { canScrollLeft, canScrollRight, update } = useHorizontalOverflow(scroller)

// Chips come and go with the notes; the scroller itself keeps its size, so re-measure.
watch(
  () => [props.tags.length, props.untaggedCount > 0],
  () => nextTick(update),
)

function toggle(key: string) {
  selected.value = selected.value.includes(key)
    ? selected.value.filter((k) => k !== key)
    : [...selected.value, key]
}

const chip =
  'inline-flex h-6 shrink-0 items-center gap-1.5 rounded-full border px-2.5 text-xs outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring/40'
const idle = 'bg-card text-muted-foreground hover:bg-accent hover:text-foreground'
const on = 'border-foreground/30 bg-accent text-foreground font-medium'
</script>

<template>
  <div class="relative">
    <div ref="scroller" class="flex gap-1 overflow-x-auto [scrollbar-width:none]">
      <button
        type="button"
        :class="cn(chip, selected.length === 0 ? on : idle)"
        title="顯示所有便籤"
        @click="selected = []"
      >
        全部
      </button>
      <button
        v-for="tag in tags"
        :key="tag.key"
        type="button"
        :class="cn(chip, selected.includes(tag.key) ? on : idle)"
        :aria-pressed="selected.includes(tag.key)"
        @click="toggle(tag.key)"
      >
        <span class="size-1.5 rounded-full" :style="{ backgroundColor: tagColorValue(tag.name) }" />
        <span class="max-w-32 truncate">{{ tag.name }}</span>
        <span class="text-[11px] font-normal text-muted-foreground tabular-nums">{{ tag.count }}</span>
      </button>
      <button
        v-if="untaggedCount > 0"
        type="button"
        :class="cn(chip, selected.includes(UNTAGGED_KEY) ? on : idle)"
        :aria-pressed="selected.includes(UNTAGGED_KEY)"
        @click="toggle(UNTAGGED_KEY)"
      >
        未分類
        <span class="text-[11px] font-normal text-muted-foreground tabular-nums">{{ untaggedCount }}</span>
      </button>
    </div>
    <!-- Edge fades hint that more chips are scrolled out of view (mouse wheel scrolls sideways). -->
    <div
      v-if="canScrollLeft"
      class="pointer-events-none absolute inset-y-0 left-0 w-6 bg-linear-to-r from-card to-transparent"
    />
    <div
      v-if="canScrollRight"
      class="pointer-events-none absolute inset-y-0 right-0 w-6 bg-linear-to-l from-card to-transparent"
    />
  </div>
</template>
