<script setup lang="ts">
import type { Component } from 'vue'
import type { Editor } from '@tiptap/vue-3'
import {
  Bold,
  Code,
  Heading1,
  Heading2,
  Highlighter,
  Italic,
  List,
  ListChecks,
  ListOrdered,
  SquareCode,
  Strikethrough,
  TextQuote,
  Underline,
} from 'lucide-vue-next'

import { toggleMergedCodeBlock } from '@/lib/noteEditor'
import { cn } from '@/lib/utils'

const props = defineProps<{ editor: Editor }>()

interface ToolbarItem {
  title: string
  icon: Component
  isActive: () => boolean
  run: () => void
}

const chain = () => props.editor.chain().focus()

/** `null` renders a separator. */
const items: (ToolbarItem | null)[] = [
  { title: '粗體 (Ctrl+B)', icon: Bold, isActive: () => props.editor.isActive('bold'), run: () => chain().toggleBold().run() },
  { title: '斜體 (Ctrl+I)', icon: Italic, isActive: () => props.editor.isActive('italic'), run: () => chain().toggleItalic().run() },
  { title: '底線 (Ctrl+U)', icon: Underline, isActive: () => props.editor.isActive('underline'), run: () => chain().toggleUnderline().run() },
  { title: '刪除線 (Ctrl+Shift+S)', icon: Strikethrough, isActive: () => props.editor.isActive('strike'), run: () => chain().toggleStrike().run() },
  { title: '螢光標記 (Ctrl+Shift+H)', icon: Highlighter, isActive: () => props.editor.isActive('highlight'), run: () => chain().toggleHighlight().run() },
  null,
  { title: '標題 1（# 空白）', icon: Heading1, isActive: () => props.editor.isActive('heading', { level: 1 }), run: () => chain().toggleHeading({ level: 1 }).run() },
  { title: '標題 2（## 空白）', icon: Heading2, isActive: () => props.editor.isActive('heading', { level: 2 }), run: () => chain().toggleHeading({ level: 2 }).run() },
  null,
  { title: '項目清單（- 空白）', icon: List, isActive: () => props.editor.isActive('bulletList'), run: () => chain().toggleBulletList().run() },
  { title: '編號清單（1. 空白）', icon: ListOrdered, isActive: () => props.editor.isActive('orderedList'), run: () => chain().toggleOrderedList().run() },
  { title: '核取清單（[ ] 空白）', icon: ListChecks, isActive: () => props.editor.isActive('taskList'), run: () => chain().toggleTaskList().run() },
  null,
  { title: '行內程式碼：套用到選取文字 (Ctrl+E)', icon: Code, isActive: () => props.editor.isActive('code'), run: () => chain().toggleCode().run() },
  { title: '程式碼區塊：整段（```）', icon: SquareCode, isActive: () => props.editor.isActive('codeBlock'), run: () => toggleMergedCodeBlock(props.editor) },
  { title: '引用（> 空白）', icon: TextQuote, isActive: () => props.editor.isActive('blockquote'), run: () => chain().toggleBlockquote().run() },
]
</script>

<template>
  <!-- mousedown.prevent keeps the text selection while clicking a button. -->
  <div
    class="flex h-7 shrink-0 items-center gap-px overflow-x-auto border-t border-note-border/50 px-1 [scrollbar-width:none]"
    @mousedown.prevent
  >
    <template v-for="(item, index) in items" :key="index">
      <span v-if="item === null" class="mx-0.5 h-3.5 w-px shrink-0 bg-note-ink/15" />
      <button
        v-else
        type="button"
        :title="item.title"
        :class="
          cn(
            'inline-flex size-6 shrink-0 items-center justify-center rounded-sm text-note-ink/60 outline-none hover:bg-note-ink/10 hover:text-note-ink',
            item.isActive() && 'bg-note-ink/15 text-note-ink',
          )
        "
        @click="item.run()"
      >
        <component :is="item.icon" class="size-3.5" />
      </button>
    </template>
  </div>
</template>
