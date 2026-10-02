<script setup lang="ts">
import { computed } from 'vue'
import { EyeOff, Pin } from 'lucide-vue-next'

import NoteItemMenu from '@/components/main/NoteItemMenu.vue'
import { NOTE_COLOR_CLASSES } from '@/constants/noteColors'
import { cn } from '@/lib/utils'
import type { Note } from '@/models/note'
import { markdownToPlainText } from '@/utils/markdown'
import { formatNoteTime } from '@/utils/time'

const props = defineProps<{ note: Note }>()

const emit = defineEmits<{
  open: []
  archive: []
  restore: []
  delete: []
  'delete-permanently': []
}>()

const lines = computed(() =>
  markdownToPlainText(props.note.content)
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean),
)

/** Untitled notes use their first line as the heading, like Windows Sticky Notes. */
const heading = computed(() => props.note.title.trim() || lines.value[0] || '空白便籤')
const preview = computed(() => (props.note.title.trim() ? lines.value : lines.value.slice(1)).join('  '))

const timeLabel = computed(() =>
  props.note.status === 'deleted' && props.note.deletedAt
    ? `刪除於 ${formatNoteTime(props.note.deletedAt)}`
    : formatNoteTime(props.note.updatedAt),
)

const isActive = computed(() => props.note.status === 'active')
/** Active, but its window was closed with × — not on the desktop right now. */
const isHidden = computed(() => isActive.value && !props.note.isOpen)
</script>

<template>
  <div
    :class="
      cn(
        NOTE_COLOR_CLASSES[note.color],
        'group flex gap-2.5 rounded-md border bg-card py-2.5 pr-1.5 pl-2.5 transition-colors',
        isActive && 'hover:border-ring/50 hover:bg-note-body/40',
        isHidden && 'bg-muted/50',
      )
    "
    :role="isActive ? 'button' : undefined"
    :tabindex="isActive ? 0 : undefined"
    @click="isActive && emit('open')"
    @keydown.enter="isActive && emit('open')"
  >
    <span class="w-1 shrink-0 self-stretch rounded-full bg-note-border" />

    <div class="min-w-0 flex-1">
      <div class="truncate text-[13px] font-semibold">{{ heading }}</div>
      <p v-if="preview" class="mt-0.5 line-clamp-2 text-xs break-all text-muted-foreground">{{ preview }}</p>
      <div class="mt-1 flex items-center gap-1.5 text-[11px] text-muted-foreground/80">
        <span>{{ timeLabel }}</span>
        <Pin v-if="note.alwaysOnTop && isActive" class="size-3 fill-current" />
        <span v-if="isHidden" class="inline-flex items-center gap-0.5" title="這張便籤已關閉，點擊即可重新顯示在桌面">
          <EyeOff class="size-3" />
          未顯示
        </span>
      </div>
    </div>

    <!-- In the normal flow (not absolutely positioned) so it can never cover the text. -->
    <div class="shrink-0 self-start">
      <NoteItemMenu
        :status="note.status"
        @open="emit('open')"
        @archive="emit('archive')"
        @restore="emit('restore')"
        @delete="emit('delete')"
        @delete-permanently="emit('delete-permanently')"
      />
    </div>
  </div>
</template>
