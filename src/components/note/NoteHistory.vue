<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { ArrowLeft, History, RotateCcw } from 'lucide-vue-next'

import { cn } from '@/lib/utils'
import type { Note, NoteRevision } from '@/models/note'
import { useNoteStore } from '@/stores/note'
import { markdownToPlainText } from '@/utils/markdown'
import { noteHeading } from '@/utils/noteText'
import { formatNoteTime } from '@/utils/time'

/**
 * Earlier versions of the note, shown in place of the editor. A version is kept
 * at most every 10 minutes while editing; restoring keeps the current text as a version too.
 */
const props = defineProps<{ noteId: string }>()

const emit = defineEmits<{
  close: []
  restored: [note: Note]
}>()

const store = useNoteStore()

const revisions = ref<NoteRevision[] | null>(null)
const selectedId = ref<number | null>(null)
const restoring = ref(false)

const selected = computed(() => revisions.value?.find((r) => r.id === selectedId.value) ?? null)
const previewText = computed(() => (selected.value ? markdownToPlainText(selected.value.content).trim() : ''))

onMounted(async () => {
  revisions.value = (await store.loadRevisions(props.noteId)) ?? []
})

async function restore() {
  if (!selected.value || restoring.value) return
  restoring.value = true
  const note = await store.restoreRevision(props.noteId, selected.value.id)
  restoring.value = false
  if (note) emit('restored', note)
}

const button =
  'inline-flex h-6 items-center gap-1 rounded-sm px-1.5 text-xs outline-none hover:bg-note-ink/10 focus-visible:bg-note-ink/10'
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col text-note-ink">
    <div class="flex h-8 shrink-0 items-center gap-1 border-b border-note-border/70 px-1.5">
      <button
        type="button"
        :class="button"
        :title="selected ? '回到版本清單' : '回到便籤'"
        @click="selected ? (selectedId = null) : emit('close')"
      >
        <ArrowLeft class="size-3.5" />
        {{ selected ? '版本清單' : '返回' }}
      </button>
      <span class="ml-1 truncate text-xs text-note-ink/60">
        {{ selected ? formatNoteTime(selected.createdAt) : '歷史版本' }}
      </span>
      <button
        v-if="selected"
        type="button"
        :class="cn(button, 'ml-auto shrink-0 bg-note-ink/10 font-medium')"
        :disabled="restoring"
        @click="restore"
      >
        <RotateCcw class="size-3.5" />
        還原此版本
      </button>
    </div>

    <!-- Preview of one version -->
    <div v-if="selected" class="min-h-0 flex-1 overflow-y-auto px-3 py-2 text-[13px] leading-relaxed">
      <p v-if="selected.title" class="mb-1 font-semibold">{{ selected.title }}</p>
      <p class="break-all whitespace-pre-wrap">{{ previewText || '（空白）' }}</p>
    </div>

    <!-- List of versions -->
    <ul v-else class="min-h-0 flex-1 overflow-y-auto p-1">
      <li v-if="revisions === null" class="py-6 text-center text-xs text-note-ink/60">載入中…</li>
      <li v-else-if="revisions.length === 0" class="flex flex-col items-center gap-1.5 px-4 py-6 text-center text-xs text-note-ink/60">
        <History class="size-5 opacity-60" />
        還沒有歷史版本。編輯時每 10 分鐘會自動保留一次之前的內容。
      </li>
      <li v-for="revision in revisions ?? []" :key="revision.id">
        <button
          type="button"
          class="flex w-full flex-col items-start rounded-sm px-2 py-1.5 text-left outline-none hover:bg-note-ink/10 focus-visible:bg-note-ink/10"
          @click="selectedId = revision.id"
        >
          <span class="text-[11px] text-note-ink/60">{{ formatNoteTime(revision.createdAt) }}</span>
          <span class="w-full truncate text-[13px]">{{ noteHeading(revision) }}</span>
        </button>
      </li>
    </ul>
  </div>
</template>
