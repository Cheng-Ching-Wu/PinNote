<script setup lang="ts">
import { onMounted } from 'vue'
import { ArrowLeft } from 'lucide-vue-next'

import { useShortcutStore } from '@/stores/shortcuts'
import { formatAccelerator } from '@/utils/accelerator'

/** Keyboard reference shown in place of the editor (note menu → 快捷鍵說明). */
const emit = defineEmits<{ close: [] }>()

const shortcuts = useShortcutStore()
onMounted(() => void shortcuts.load())

interface Entry {
  keys: string
  label: string
}

/** Bindings of the Tiptap extensions in `lib/noteEditor.ts`. */
const SECTIONS: { title: string; entries: Entry[] }[] = [
  {
    title: '文字格式',
    entries: [
      { keys: 'Ctrl+B', label: '粗體' },
      { keys: 'Ctrl+I', label: '斜體' },
      { keys: 'Ctrl+U', label: '底線' },
      { keys: 'Ctrl+Shift+S', label: '刪除線' },
      { keys: 'Ctrl+Shift+H', label: '螢光標記' },
      { keys: 'Ctrl+E', label: '行內程式碼' },
    ],
  },
  {
    title: '段落',
    entries: [
      { keys: 'Ctrl+Alt+1 / 2 / 3', label: '標題 1 / 2 / 3' },
      { keys: 'Ctrl+Shift+8', label: '項目清單' },
      { keys: 'Ctrl+Shift+7', label: '編號清單' },
      { keys: 'Ctrl+Shift+9', label: '待辦清單' },
      { keys: 'Ctrl+Shift+B', label: '引用' },
      { keys: 'Ctrl+Alt+C', label: '程式碼區塊' },
      { keys: 'Tab / Shift+Tab', label: '清單縮排 / 取消縮排' },
      { keys: 'Shift+Enter', label: '換行（不分段）' },
    ],
  },
  {
    title: 'Markdown 快速輸入（行首輸入後加空白）',
    entries: [
      { keys: '# / ## / ###', label: '標題' },
      { keys: '- 或 *', label: '項目清單' },
      { keys: '1.', label: '編號清單' },
      { keys: '[ ]', label: '待辦項目' },
      { keys: '>', label: '引用' },
      { keys: '```', label: '程式碼區塊' },
    ],
  },
  {
    title: '便籤',
    entries: [
      { keys: '[[', label: '連結其他便籤（↑↓ 選擇、Enter 插入）' },
      { keys: 'Ctrl+點擊', label: '開啟連結（網頁或便籤）' },
      { keys: 'Enter（標題）', label: '跳到內文' },
      { keys: 'Ctrl+Z / Ctrl+Y', label: '復原 / 重做' },
    ],
  },
]
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col text-note-ink">
    <div class="flex h-8 shrink-0 items-center gap-1 border-b border-note-border/70 px-1.5">
      <button
        type="button"
        class="inline-flex h-6 items-center gap-1 rounded-sm px-1.5 text-xs outline-none hover:bg-note-ink/10 focus-visible:bg-note-ink/10"
        title="回到便籤"
        @click="emit('close')"
      >
        <ArrowLeft class="size-3.5" />
        返回
      </button>
      <span class="ml-1 truncate text-xs text-note-ink/60">快捷鍵說明</span>
    </div>

    <div class="min-h-0 flex-1 space-y-3 overflow-y-auto px-3 py-2 text-xs">
      <section>
        <h3 class="mb-1 font-semibold">全域快捷鍵</h3>
        <dl class="space-y-0.5">
          <div v-for="item in shortcuts.statuses" :key="item.action" class="flex items-baseline gap-2">
            <dt class="w-28 shrink-0">
              <kbd v-if="item.accelerator" class="rounded-sm bg-note-ink/10 px-1 py-px font-sans">
                {{ formatAccelerator(item.accelerator) }}
              </kbd>
              <span v-else class="text-note-ink/50">未設定</span>
            </dt>
            <dd class="min-w-0">{{ item.label }}</dd>
          </div>
        </dl>
        <p class="mt-1 text-[11px] text-note-ink/60">可在便籤管理的「設定 → 自訂快捷鍵」修改。</p>
      </section>

      <section v-for="section in SECTIONS" :key="section.title">
        <h3 class="mb-1 font-semibold">{{ section.title }}</h3>
        <dl class="space-y-0.5">
          <div v-for="entry in section.entries" :key="entry.keys" class="flex items-baseline gap-2">
            <dt class="w-28 shrink-0">
              <kbd class="rounded-sm bg-note-ink/10 px-1 py-px font-sans">{{ entry.keys }}</kbd>
            </dt>
            <dd class="min-w-0">{{ entry.label }}</dd>
          </div>
        </dl>
      </section>
    </div>
  </div>
</template>
