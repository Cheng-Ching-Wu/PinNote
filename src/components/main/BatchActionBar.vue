<script setup lang="ts">
import { ref, watch } from 'vue'
import { Archive, ArchiveRestore, Palette, RotateCcw, Tag, Trash2, X } from 'lucide-vue-next'

import { Button, buttonVariants } from '@/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { NOTE_COLOR_CLASSES, NOTE_COLOR_LABELS, NOTE_COLORS } from '@/constants/noteColors'
import { cn } from '@/lib/utils'
import type { NoteColor, NoteStatus } from '@/models/note'

const props = defineProps<{
  count: number
  allSelected: boolean
  /** Status of the current tab; decides which actions make sense. */
  status: NoteStatus
}>()

const emit = defineEmits<{
  'select-all': []
  /** Deselect everything (batch mode stays on). */
  clear: []
  /** Turn batch mode off. */
  exit: []
  color: [color: NoteColor]
  tags: []
  archive: []
  restore: []
  delete: []
  'delete-permanently': []
}>()

/** Permanent deletion needs a second click; reset when the selection changes. */
const confirming = ref(false)
watch(() => props.count, () => (confirming.value = false))

function deletePermanently() {
  if (!confirming.value) {
    confirming.value = true
    return
  }
  confirming.value = false
  emit('delete-permanently')
}

const action = cn(buttonVariants({ variant: 'ghost', size: 'sm' }), 'h-7 px-2 text-xs')
</script>

<template>
  <div class="border-t bg-card px-3 py-2">
    <div class="flex items-center gap-2 text-xs">
      <span class="font-medium tabular-nums">
        {{ count > 0 ? `已選取 ${count} 張` : '點擊便籤以選取' }}
      </span>
      <button
        type="button"
        class="shrink-0 text-muted-foreground underline-offset-2 hover:underline"
        @click="allSelected ? emit('clear') : emit('select-all')"
      >
        {{ allSelected ? '取消全選' : '全選' }}
      </button>
      <Button variant="ghost" size="icon-xs" title="結束批次操作 (Esc)" class="ml-auto" @click="emit('exit')">
        <X class="size-3.5" />
      </Button>
    </div>

    <!-- Actions stay visible (disabled) with nothing selected, so the bar doesn't jump. -->
    <fieldset :disabled="count === 0" class="mt-1.5 flex flex-wrap items-center gap-0.5 disabled:opacity-50">
      <template v-if="status !== 'deleted'">
        <DropdownMenu :modal="false">
          <DropdownMenuTrigger :class="cn(action, 'data-[state=open]:bg-accent')" title="變更顏色">
            <Palette class="size-3.5" />
            顏色
          </DropdownMenuTrigger>
          <DropdownMenuContent class="w-auto" align="start" side="top">
            <div class="flex items-center gap-2 p-1.5">
              <button
                v-for="option in NOTE_COLORS"
                :key="option"
                type="button"
                :title="NOTE_COLOR_LABELS[option]"
                :class="
                  cn(
                    NOTE_COLOR_CLASSES[option],
                    'size-5 rounded-full border border-note-border bg-note-header transition-transform hover:scale-110',
                  )
                "
                @click="emit('color', option)"
              />
            </div>
          </DropdownMenuContent>
        </DropdownMenu>
        <button type="button" :class="action" @click="emit('tags')">
          <Tag class="size-3.5" />
          標籤
        </button>
        <button v-if="status === 'active'" type="button" :class="action" @click="emit('archive')">
          <Archive class="size-3.5" />
          封存
        </button>
        <button v-else type="button" :class="action" @click="emit('restore')">
          <ArchiveRestore class="size-3.5" />
          取消封存
        </button>
        <button
          type="button"
          :class="cn(action, 'ml-auto text-destructive hover:bg-destructive/10 hover:text-destructive')"
          @click="emit('delete')"
        >
          <Trash2 class="size-3.5" />
          移到垃圾桶
        </button>
      </template>

      <template v-else>
        <button type="button" :class="action" @click="emit('restore')">
          <RotateCcw class="size-3.5" />
          恢復
        </button>
        <button
          type="button"
          :class="
            cn(
              action,
              'ml-auto text-destructive hover:bg-destructive/10 hover:text-destructive',
              confirming && 'bg-destructive/10 font-medium',
            )
          "
          @click="deletePermanently"
        >
          <Trash2 class="size-3.5" />
          {{ confirming ? '再按一次確認刪除' : '永久刪除' }}
        </button>
      </template>
    </fieldset>
  </div>
</template>
