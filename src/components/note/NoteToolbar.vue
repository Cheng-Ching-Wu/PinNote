<script setup lang="ts">
import { type Component, ref, useTemplateRef } from 'vue'
import type { Editor } from '@tiptap/vue-3'
import {
  Bold,
  ChevronLeft,
  ChevronRight,
  Code,
  Heading1,
  Heading2,
  Highlighter,
  ImagePlus,
  Italic,
  List,
  ListChecks,
  ListOrdered,
  SquareCode,
  Strikethrough,
  TextQuote,
  Underline,
} from 'lucide-vue-next'

import NoteTableMenu from '@/components/note/NoteTableMenu.vue'
import { useHorizontalOverflow } from '@/composables/useHorizontalOverflow'
import { type MessageKey, t } from '@/i18n'
import { toggleMergedCodeBlock } from '@/lib/noteEditor'
import { cn } from '@/lib/utils'
import { IMAGE_MIME_TYPES } from '@/services/imageService'

const props = defineProps<{ editor: Editor }>()

const emit = defineEmits<{ 'insert-images': [files: File[]] }>()

const buttonClass =
  'inline-flex size-6 shrink-0 items-center justify-center rounded-sm text-note-ink/60 outline-none hover:bg-note-ink/10 hover:text-note-ink data-[state=open]:bg-note-ink/15'
const activeClass = 'bg-note-ink/15 text-note-ink'
const separatorClass = 'mx-0.5 h-3.5 w-px shrink-0 bg-note-ink/15'

// Rounded buttons in the note's own palette, floating over the scroller edges; buttons slide underneath them.
const arrowClass =
  'absolute top-1/2 z-10 inline-flex size-5 -translate-y-1/2 items-center justify-center rounded-md border border-note-border bg-note-header text-note-ink outline-none transition-colors hover:bg-note-border'

const fileInput = useTemplateRef<HTMLInputElement>('fileInput')
const scroller = useTemplateRef<HTMLDivElement>('scroller')
const { canScrollLeft, canScrollRight, scrollByPage, startAutoScroll, stopAutoScroll } =
  useHorizontalOverflow(scroller)

const barHovered = ref(false)

function onBarLeave() {
  barHovered.value = false
  stopAutoScroll()
}

function onFilesPicked(event: Event) {
  const input = event.target as HTMLInputElement
  const files = Array.from(input.files ?? [])
  input.value = '' // allow picking the same file again
  if (files.length > 0) emit('insert-images', files)
}

interface ToolbarItem {
  title: MessageKey
  icon: Component
  isActive: () => boolean
  run: () => void
}

const chain = () => props.editor.chain().focus()

/** `null` renders a separator. */
const items: (ToolbarItem | null)[] = [
  { title: 'toolbar.bold', icon: Bold, isActive: () => props.editor.isActive('bold'), run: () => chain().toggleBold().run() },
  { title: 'toolbar.italic', icon: Italic, isActive: () => props.editor.isActive('italic'), run: () => chain().toggleItalic().run() },
  { title: 'toolbar.underline', icon: Underline, isActive: () => props.editor.isActive('underline'), run: () => chain().toggleUnderline().run() },
  { title: 'toolbar.strike', icon: Strikethrough, isActive: () => props.editor.isActive('strike'), run: () => chain().toggleStrike().run() },
  { title: 'toolbar.highlight', icon: Highlighter, isActive: () => props.editor.isActive('highlight'), run: () => chain().toggleHighlight().run() },
  null,
  { title: 'toolbar.heading1', icon: Heading1, isActive: () => props.editor.isActive('heading', { level: 1 }), run: () => chain().toggleHeading({ level: 1 }).run() },
  { title: 'toolbar.heading2', icon: Heading2, isActive: () => props.editor.isActive('heading', { level: 2 }), run: () => chain().toggleHeading({ level: 2 }).run() },
  null,
  { title: 'toolbar.bulletList', icon: List, isActive: () => props.editor.isActive('bulletList'), run: () => chain().toggleBulletList().run() },
  { title: 'toolbar.orderedList', icon: ListOrdered, isActive: () => props.editor.isActive('orderedList'), run: () => chain().toggleOrderedList().run() },
  { title: 'toolbar.taskList', icon: ListChecks, isActive: () => props.editor.isActive('taskList'), run: () => chain().toggleTaskList().run() },
  null,
  { title: 'toolbar.code', icon: Code, isActive: () => props.editor.isActive('code'), run: () => chain().toggleCode().run() },
  { title: 'toolbar.codeBlock', icon: SquareCode, isActive: () => props.editor.isActive('codeBlock'), run: () => toggleMergedCodeBlock(props.editor) },
  { title: 'toolbar.blockquote', icon: TextQuote, isActive: () => props.editor.isActive('blockquote'), run: () => chain().toggleBlockquote().run() },
]
</script>

<template>
  <!-- mousedown.prevent keeps the text selection while clicking a button. -->
  <div
    class="relative flex h-7 shrink-0 items-center border-t border-note-border/50"
    @mousedown.prevent
    @pointerenter="barHovered = true"
    @pointerleave="onBarLeave"
  >
    <!--
      Arrows appear only while the pointer is on the toolbar and buttons are cut off
      (e.g. the note was resized narrower). Resting on an arrow scrolls; clicking pages.
    -->
    <button
      v-if="barHovered && canScrollLeft"
      type="button"
      :class="cn(arrowClass, 'left-1')"
      :title="t('toolbar.scrollLeft')"
      @pointerenter="startAutoScroll(-1)"
      @pointerleave="stopAutoScroll"
      @click="scrollByPage(-1)"
    >
      <ChevronLeft class="size-3" :stroke-width="3" />
    </button>

    <div
      ref="scroller"
      class="flex h-full min-w-0 flex-1 items-center gap-px overflow-x-auto px-1 [scrollbar-width:none]"
    >
      <template v-for="(item, index) in items" :key="index">
        <span v-if="item === null" :class="separatorClass" />
        <button
          v-else
          type="button"
          :title="t(item.title)"
          :class="cn(buttonClass, item.isActive() && activeClass)"
          @click="item.run()"
        >
          <component :is="item.icon" class="size-3.5" />
        </button>
      </template>

      <span :class="separatorClass" />
      <NoteTableMenu :editor="editor" :button-class="buttonClass" :active-class="activeClass" />
      <button type="button" :class="buttonClass" :title="t('toolbar.image')" @click="fileInput?.click()">
        <ImagePlus class="size-3.5" />
      </button>
    </div>

    <button
      v-if="barHovered && canScrollRight"
      type="button"
      :class="cn(arrowClass, 'right-1')"
      :title="t('toolbar.scrollRight')"
      @pointerenter="startAutoScroll(1)"
      @pointerleave="stopAutoScroll"
      @click="scrollByPage(1)"
    >
      <ChevronRight class="size-3" :stroke-width="3" />
    </button>

    <input
      ref="fileInput"
      type="file"
      :accept="IMAGE_MIME_TYPES.join(',')"
      multiple
      class="hidden"
      @change="onFilesPicked"
    />
  </div>
</template>
