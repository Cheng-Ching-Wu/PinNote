<script setup lang="ts">
import { Archive, Ellipsis, History, Keyboard, LayoutList, Plus, Tag, Trash2 } from 'lucide-vue-next'

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { NOTE_COLOR_CLASSES, NOTE_COLOR_LABELS, NOTE_COLORS } from '@/constants/noteColors'
import { cn } from '@/lib/utils'
import type { NoteColor } from '@/models/note'

defineProps<{ color: NoteColor }>()

const emit = defineEmits<{
  'set-color': [color: NoteColor]
  'edit-tags': []
  history: []
  shortcuts: []
  'new-note': []
  'open-main': []
  archive: []
  delete: []
}>()

/**
 * "新增標籤" moves focus into the tag input. Without this the closing menu hands
 * focus back to its trigger, the input blurs and tag editing ends immediately.
 */
let keepFocus = false

function editTags() {
  keepFocus = true
  emit('edit-tags')
}

function onCloseAutoFocus(event: Event) {
  if (!keepFocus) return
  keepFocus = false
  event.preventDefault()
}
</script>

<template>
  <DropdownMenu :modal="false">
    <DropdownMenuTrigger
      class="inline-flex size-6 items-center justify-center rounded-sm text-note-ink/70 outline-none hover:bg-note-ink/10 hover:text-note-ink data-[state=open]:bg-note-ink/10"
      title="更多"
    >
      <Ellipsis class="size-3.5" />
    </DropdownMenuTrigger>

    <DropdownMenuContent class="w-44" @close-auto-focus="onCloseAutoFocus">
      <div class="flex items-center justify-between px-1.5 py-1.5">
        <button
          v-for="option in NOTE_COLORS"
          :key="option"
          type="button"
          :title="NOTE_COLOR_LABELS[option]"
          :class="
            cn(
              NOTE_COLOR_CLASSES[option],
              'size-5 rounded-full border border-note-border bg-note-header transition-transform hover:scale-110',
              option === color && 'ring-2 ring-ring/60 ring-offset-1 ring-offset-card',
            )
          "
          @click="emit('set-color', option)"
        />
      </div>
      <DropdownMenuSeparator />
      <DropdownMenuItem @select="editTags">
        <Tag />
        新增標籤
      </DropdownMenuItem>
      <DropdownMenuItem @select="emit('history')">
        <History />
        歷史版本
      </DropdownMenuItem>
      <DropdownMenuItem @select="emit('new-note')">
        <Plus />
        新增便籤
      </DropdownMenuItem>
      <DropdownMenuItem @select="emit('open-main')">
        <LayoutList />
        便籤管理
      </DropdownMenuItem>
      <DropdownMenuItem @select="emit('shortcuts')">
        <Keyboard />
        快捷鍵說明
      </DropdownMenuItem>
      <DropdownMenuSeparator />
      <DropdownMenuItem @select="emit('archive')">
        <Archive />
        封存
      </DropdownMenuItem>
      <DropdownMenuItem variant="destructive" @select="emit('delete')">
        <Trash2 />
        刪除
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
