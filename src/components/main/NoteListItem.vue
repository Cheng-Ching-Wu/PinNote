<script setup lang="ts">
import { computed } from 'vue'
import { Check, EyeOff, GripVertical, Pin } from 'lucide-vue-next'

import NoteItemMenu from '@/components/main/NoteItemMenu.vue'
import TagChip from '@/components/tags/TagChip.vue'
import { useTagColors } from '@/composables/useTagColors'
import { NOTE_COLOR_CLASSES } from '@/constants/noteColors'
import { t } from '@/i18n'
import { cn } from '@/lib/utils'
import type { Note } from '@/models/note'
import { noteHeading, noteLines, notePreview } from '@/utils/noteText'
import { formatNoteTime } from '@/utils/time'

const props = defineProps<{
  note: Note
  /** Batch mode is on: the item shows a check circle and clicks select it. */
  selecting?: boolean
  selected?: boolean
  /** Manual sorting: shows a drag handle. The drag events themselves come from the parent. */
  reorderable?: boolean
  /** Where a dragged note would land relative to this one. */
  dropPosition?: 'before' | 'after' | null
}>()

const emit = defineEmits<{
  /** Click / Enter on the item; the parent decides between opening and selecting. */
  activate: [event: MouseEvent | KeyboardEvent]
  open: []
  'edit-tags': []
  archive: []
  restore: []
  delete: []
  'delete-permanently': []
}>()

const lines = computed(() => noteLines(props.note))
const heading = computed(() => noteHeading(props.note, lines.value))
const preview = computed(() => notePreview(props.note, lines.value))

const timeLabel = computed(() =>
  props.note.status === 'deleted' && props.note.deletedAt
    ? t('item.deletedAt', { time: formatNoteTime(props.note.deletedAt) })
    : formatNoteTime(props.note.updatedAt),
)

const { tagColorValue } = useTagColors()

/** Only a few chips fit on the tag row; the rest are summarised as "+N". */
const MAX_VISIBLE_TAGS = 3
const visibleTags = computed(() => props.note.tags.slice(0, MAX_VISIBLE_TAGS))
const hiddenTags = computed(() => props.note.tags.slice(MAX_VISIBLE_TAGS))

const isActive = computed(() => props.note.status === 'active')
/** Active, but its window was closed with × — not on the desktop right now. */
const isHidden = computed(() => isActive.value && !props.note.isOpen)
/** Archived / trashed notes can't be opened, but can still be selected. */
const clickable = computed(() => isActive.value || props.selecting)

/** Shift+click selects a range in batch mode — don't let it select page text too. */
function onMousedown(event: MouseEvent) {
  if (props.selecting && event.shiftKey) event.preventDefault()
}
</script>

<template>
  <div
    :class="
      cn(
        NOTE_COLOR_CLASSES[note.color],
        'group relative flex gap-2.5 rounded-md border bg-card py-2.5 pr-1.5 pl-2.5 transition-colors',
        clickable && 'hover:border-ring/50 hover:bg-note-body/40',
        isHidden && 'bg-muted/50',
        selected && 'border-ring/70 bg-note-body/50 ring-1 ring-ring/40 hover:bg-note-body/60',
      )
    "
    role="button"
    tabindex="0"
    :aria-pressed="selecting ? selected : undefined"
    @mousedown="onMousedown"
    @click="emit('activate', $event)"
    @keydown.enter.self="emit('activate', $event)"
    @keydown.space.self.prevent="selecting && emit('activate', $event)"
  >
    <!-- Drop indicator for manual sorting (sits in the gap between items). -->
    <span
      v-if="dropPosition"
      :class="
        cn(
          'pointer-events-none absolute inset-x-1 h-0.5 rounded-full bg-primary',
          dropPosition === 'before' ? '-top-[5px]' : '-bottom-[5px]',
        )
      "
    />

    <span
      v-if="selecting"
      :class="
        cn(
          'mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-full border border-input bg-card',
          selected && 'border-primary bg-primary text-primary-foreground',
        )
      "
    >
      <Check v-if="selected" class="size-3" />
    </span>
    <span class="w-1 shrink-0 self-stretch rounded-full bg-note-border" />

    <div class="min-w-0 flex-1">
      <div class="truncate text-[13px] font-semibold">{{ heading }}</div>
      <p v-if="preview" class="mt-0.5 line-clamp-2 text-xs break-all text-muted-foreground">{{ preview }}</p>
      <div class="mt-1 flex min-w-0 items-center gap-1.5 text-[11px] text-muted-foreground/80">
        <span class="shrink-0">{{ timeLabel }}</span>
        <Pin v-if="note.alwaysOnTop && isActive" class="size-3 fill-current" />
        <span v-if="isHidden" class="inline-flex items-center gap-0.5" :title="t('item.hiddenTitle')">
          <EyeOff class="size-3" />
          {{ t('item.hidden') }}
        </span>
      </div>
      <div
        v-if="note.tags.length > 0"
        class="mt-1 flex min-w-0 items-center gap-1 overflow-hidden text-[11px] text-foreground/80"
      >
        <TagChip v-for="tag in visibleTags" :key="tag" :name="tag" :color="tagColorValue(tag)" class="max-w-24" />
        <span v-if="hiddenTags.length" class="shrink-0 text-muted-foreground" :title="hiddenTags.join(t('item.tagSeparator'))">
          +{{ hiddenTags.length }}
        </span>
      </div>
    </div>

    <!-- In the normal flow (not absolutely positioned) so it can never cover the text. -->
    <div class="flex shrink-0 flex-col items-center self-stretch">
      <NoteItemMenu
        :status="note.status"
        @open="emit('open')"
        @edit-tags="emit('edit-tags')"
        @archive="emit('archive')"
        @restore="emit('restore')"
        @delete="emit('delete')"
        @delete-permanently="emit('delete-permanently')"
      />
      <GripVertical
        v-if="reorderable"
        class="mt-auto size-3.5 cursor-grab text-muted-foreground/50 opacity-0 group-hover:opacity-100"
        aria-hidden="true"
      />
    </div>
  </div>
</template>
