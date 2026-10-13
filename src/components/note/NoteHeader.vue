<script setup lang="ts">
import { Minus, Pin, PinOff, Plus, X } from 'lucide-vue-next'

import NoteMenu from '@/components/note/NoteMenu.vue'
import { cn } from '@/lib/utils'
import type { NoteColor } from '@/models/note'

defineProps<{
  color: NoteColor
  collapsed: boolean
  alwaysOnTop: boolean
}>()

const title = defineModel<string>('title', { required: true })

const emit = defineEmits<{
  'title-enter': []
  'toggle-pin': []
  'toggle-collapse': []
  close: []
  'set-color': [color: NoteColor]
  'edit-tags': []
  history: []
  shortcuts: []
  'new-note': []
  'open-main': []
  archive: []
  delete: []
}>()

const iconButton =
  'inline-flex size-6 shrink-0 items-center justify-center rounded-sm text-note-ink/70 outline-none hover:bg-note-ink/10 hover:text-note-ink focus-visible:bg-note-ink/10'
/** Revealed while the pointer is over the note (or a control inside has focus). */
const onHover = 'opacity-0 transition-opacity group-hover/note:opacity-100 focus-visible:opacity-100'
</script>

<template>
  <!-- Height must equal COLLAPSED_HEIGHT (32px). Only the header and spacer are drag regions. -->
  <header
    data-tauri-drag-region
    :class="
      cn(
        'flex h-8 shrink-0 items-center gap-0.5 bg-note-header pr-1 pl-2.5 select-none',
        !collapsed && 'border-b border-note-border/70',
      )
    "
  >
    <input
      v-model="title"
      placeholder="標題"
      spellcheck="false"
      class="h-6 max-w-[62%] min-w-12 truncate bg-transparent text-[13px] font-semibold text-note-ink outline-none [field-sizing:content] placeholder:font-normal placeholder:text-note-ink/40"
      @keydown.enter.prevent="emit('title-enter')"
    />
    <div data-tauri-drag-region class="h-full min-w-4 flex-1" />

    <div v-if="!collapsed" :class="cn('flex', onHover, 'focus-within:opacity-100 has-data-[state=open]:opacity-100')">
      <NoteMenu
        :color="color"
        @set-color="emit('set-color', $event)"
        @edit-tags="emit('edit-tags')"
        @history="emit('history')"
        @shortcuts="emit('shortcuts')"
        @new-note="emit('new-note')"
        @open-main="emit('open-main')"
        @archive="emit('archive')"
        @delete="emit('delete')"
      />
    </div>

    <button
      type="button"
      :title="alwaysOnTop ? '取消置頂' : '置頂'"
      :class="cn(iconButton, !alwaysOnTop && onHover, alwaysOnTop && 'text-note-ink')"
      @click="emit('toggle-pin')"
    >
      <Pin v-if="alwaysOnTop" class="size-3.5 fill-current" />
      <PinOff v-else class="size-3.5" />
    </button>

    <button
      type="button"
      :title="collapsed ? '展開' : '收合'"
      :class="iconButton"
      @click="emit('toggle-collapse')"
    >
      <Plus v-if="collapsed" class="size-3.5" />
      <Minus v-else class="size-3.5" />
    </button>

    <button type="button" title="關閉視窗（不會刪除）" :class="iconButton" @click="emit('close')">
      <X class="size-3.5" />
    </button>
  </header>
</template>
