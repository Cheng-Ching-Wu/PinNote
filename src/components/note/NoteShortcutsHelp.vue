<script setup lang="ts">
import { onMounted } from 'vue'
import { ArrowLeft } from 'lucide-vue-next'

import { type MessageKey, t } from '@/i18n'
import { useShortcutStore } from '@/stores/shortcuts'
import { formatAccelerator } from '@/utils/accelerator'

/** Keyboard reference shown in place of the editor (note menu → 快捷鍵說明). */
const emit = defineEmits<{ close: [] }>()

const shortcuts = useShortcutStore()
onMounted(() => void shortcuts.load())

interface Entry {
  /** Literal keys, or a message key when the keys themselves contain words. */
  keys: string | { key: MessageKey }
  label: MessageKey
}

/** Bindings of the Tiptap extensions in `lib/noteEditor.ts`. */
const SECTIONS: { title: MessageKey; entries: Entry[] }[] = [
  {
    title: 'help.sections.format',
    entries: [
      { keys: 'Ctrl+B', label: 'help.entries.bold' },
      { keys: 'Ctrl+I', label: 'help.entries.italic' },
      { keys: 'Ctrl+U', label: 'help.entries.underline' },
      { keys: 'Ctrl+Shift+S', label: 'help.entries.strike' },
      { keys: 'Ctrl+Shift+H', label: 'help.entries.highlight' },
      { keys: 'Ctrl+E', label: 'help.entries.code' },
    ],
  },
  {
    title: 'help.sections.paragraph',
    entries: [
      { keys: 'Ctrl+Alt+1 / 2 / 3', label: 'help.entries.headings' },
      { keys: 'Ctrl+Shift+8', label: 'help.entries.bulletList' },
      { keys: 'Ctrl+Shift+7', label: 'help.entries.orderedList' },
      { keys: 'Ctrl+Shift+9', label: 'help.entries.taskList' },
      { keys: 'Ctrl+Shift+B', label: 'help.entries.blockquote' },
      { keys: 'Ctrl+Alt+C', label: 'help.entries.codeBlock' },
      { keys: 'Tab / Shift+Tab', label: 'help.entries.indent' },
      { keys: 'Shift+Enter', label: 'help.entries.lineBreak' },
    ],
  },
  {
    title: 'help.sections.markdown',
    entries: [
      { keys: '# / ## / ###', label: 'help.entries.heading' },
      { keys: { key: 'help.keys.or' }, label: 'help.entries.bulletList' },
      { keys: '1.', label: 'help.entries.orderedList' },
      { keys: '[ ]', label: 'help.entries.taskItem' },
      { keys: '>', label: 'help.entries.blockquote' },
      { keys: '```', label: 'help.entries.codeBlock' },
    ],
  },
  {
    title: 'help.sections.note',
    entries: [
      { keys: '[[', label: 'help.entries.link' },
      { keys: { key: 'help.keys.ctrlClick' }, label: 'help.entries.openLink' },
      { keys: { key: 'help.keys.enterInTitle' }, label: 'help.entries.toContent' },
      { keys: 'Ctrl+Z / Ctrl+Y', label: 'help.entries.undo' },
    ],
  },
]

const keysText = (keys: Entry['keys']) => (typeof keys === 'string' ? keys : t(keys.key))
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col text-note-ink">
    <div class="flex h-8 shrink-0 items-center gap-1 border-b border-note-border/70 px-1.5">
      <button
        type="button"
        class="inline-flex h-6 items-center gap-1 rounded-sm px-1.5 text-xs outline-none hover:bg-note-ink/10 focus-visible:bg-note-ink/10"
        :title="t('common.backToNote')"
        @click="emit('close')"
      >
        <ArrowLeft class="size-3.5" />
        {{ t('common.back') }}
      </button>
      <span class="ml-1 truncate text-xs text-note-ink/60">{{ t('noteMenu.shortcuts') }}</span>
    </div>

    <div class="min-h-0 flex-1 space-y-3 overflow-y-auto px-3 py-2 text-xs">
      <section>
        <h3 class="mb-1 font-semibold">{{ t('help.global') }}</h3>
        <dl class="space-y-0.5">
          <div v-for="item in shortcuts.statuses" :key="item.action" class="flex items-baseline gap-2">
            <dt class="w-28 shrink-0">
              <kbd v-if="item.accelerator" class="rounded-sm bg-note-ink/10 px-1 py-px font-sans">
                {{ formatAccelerator(item.accelerator) }}
              </kbd>
              <span v-else class="text-note-ink/50">{{ t('common.notSet') }}</span>
            </dt>
            <dd class="min-w-0">{{ shortcuts.labelOf(item) }}</dd>
          </div>
        </dl>
        <p class="mt-1 text-[11px] text-note-ink/60">{{ t('help.globalHint') }}</p>
      </section>

      <section v-for="section in SECTIONS" :key="section.title">
        <h3 class="mb-1 font-semibold">{{ t(section.title) }}</h3>
        <dl class="space-y-0.5">
          <div v-for="entry in section.entries" :key="entry.label" class="flex items-baseline gap-2">
            <dt class="w-28 shrink-0">
              <kbd class="rounded-sm bg-note-ink/10 px-1 py-px font-sans">{{ keysText(entry.keys) }}</kbd>
            </dt>
            <dd class="min-w-0">{{ t(entry.label) }}</dd>
          </div>
        </dl>
      </section>
    </div>
  </div>
</template>
