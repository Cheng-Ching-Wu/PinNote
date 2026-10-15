<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, useTemplateRef, watch } from 'vue'
import { Archive, CornerDownLeft, Plus, Search } from 'lucide-vue-next'

import TagChip from '@/components/tags/TagChip.vue'
import { useTagColors } from '@/composables/useTagColors'
import { NOTE_COLOR_CLASSES } from '@/constants/noteColors'
import { t } from '@/i18n'
import { cn } from '@/lib/utils'
import type { Note } from '@/models/note'
import { windowService } from '@/services/windowService'
import { useNoteStore } from '@/stores/note'
import { useSettingsStore } from '@/stores/settings'
import { searchNotes } from '@/utils/search'
import { parseTagQuery } from '@/utils/tags'

/**
 * Quick search window (Ctrl+Alt+F). Kept alive by Rust and only hidden, so it
 * resets itself each time it is shown and hides when it loses focus.
 */
const store = useNoteStore()
const settings = useSettingsStore()
const { tagColorValue } = useTagColors()

const query = ref('')
const highlighted = ref(0)
const inputRef = useTemplateRef<HTMLInputElement>('inputRef')
const listRef = useTemplateRef<HTMLElement>('listRef')

const results = computed(() => searchNotes(store.notes, query.value))
/** Plain text of the query (without #tags) — offered as the title of a new note. */
const newTitle = computed(() => parseTagQuery(query.value).text.trim())
/** Results plus the "新增便籤" row. */
const rowCount = computed(() => results.value.length + (newTitle.value ? 1 : 0))

watch(query, () => (highlighted.value = 0))
watch(highlighted, async (index) => {
  await nextTick()
  listRef.value?.querySelector(`[data-row="${index}"]`)?.scrollIntoView({ block: 'nearest' })
})

function reset() {
  query.value = ''
  highlighted.value = 0
  inputRef.value?.focus()
}

async function hide() {
  await windowService.hideCurrentWindow()
}

async function open(note: Note) {
  await hide()
  // Archived notes come back to the desktop when picked here.
  if (note.status === 'archived') await store.restoreNote(note.id)
  await store.openNoteWindow(note.id)
}

async function create() {
  const title = newTitle.value
  if (!title) return
  await hide()
  await store.createNoteWithTitle(title)
}

function choose(index: number) {
  const result = results.value[index]
  if (result) void open(result.note)
  else if (index === results.value.length) void create()
}

function onKeydown(event: KeyboardEvent) {
  if (event.isComposing) return
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      if (rowCount.value) highlighted.value = (highlighted.value + 1) % rowCount.value
      break
    case 'ArrowUp':
      event.preventDefault()
      if (rowCount.value) highlighted.value = (highlighted.value - 1 + rowCount.value) % rowCount.value
      break
    case 'Enter':
      event.preventDefault()
      if (event.ctrlKey) void create()
      else choose(highlighted.value)
      break
    case 'Escape':
      event.preventDefault()
      if (query.value) query.value = ''
      else void hide()
      break
  }
}

const cleanups: (() => void)[] = []

onMounted(async () => {
  reset()
  cleanups.push(await windowService.onSearchShown(reset))
  cleanups.push(await windowService.onFocusChanged((focused) => !focused && void hide()))
  await Promise.all([store.loadNotes(), settings.loadSettings()])
})

onBeforeUnmount(() => cleanups.forEach((cleanup) => cleanup()))
</script>

<template>
  <div class="flex h-screen flex-col overflow-hidden border bg-card text-foreground" @keydown="onKeydown">
    <div data-tauri-drag-region class="flex items-center gap-2 border-b px-3.5">
      <Search class="size-4 shrink-0 text-muted-foreground" />
      <input
        ref="inputRef"
        v-model="query"
        :placeholder="t('search.placeholder')"
        spellcheck="false"
        class="h-12 min-w-0 flex-1 bg-transparent text-[15px] outline-none placeholder:text-muted-foreground"
      />
    </div>

    <ul ref="listRef" class="min-h-0 flex-1 overflow-y-auto p-1.5">
      <li
        v-for="(result, index) in results"
        :key="result.note.id"
        :data-row="index"
        :class="
          cn(
            NOTE_COLOR_CLASSES[result.note.color],
            'flex cursor-pointer gap-2.5 rounded-md px-2.5 py-2',
            index === highlighted && 'bg-accent',
          )
        "
        @mousemove="highlighted = index"
        @click="choose(index)"
      >
        <span class="w-1 shrink-0 self-stretch rounded-full bg-note-border" />
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-1.5">
            <span class="truncate text-[13px] font-semibold">{{ result.heading }}</span>
            <span
              v-if="result.note.status === 'archived'"
              class="inline-flex shrink-0 items-center gap-0.5 text-[11px] text-muted-foreground"
              :title="t('search.archivedTitle')"
            >
              <Archive class="size-3" />
              {{ t('common.archive') }}
            </span>
          </div>
          <p v-if="result.snippet" class="truncate text-xs text-muted-foreground">{{ result.snippet }}</p>
          <div v-if="result.note.tags.length" class="mt-1 flex gap-1 overflow-hidden">
            <TagChip
              v-for="tag in result.note.tags.slice(0, 4)"
              :key="tag"
              :name="tag"
              :color="tagColorValue(tag)"
              class="max-w-28 text-muted-foreground"
            />
          </div>
        </div>
        <CornerDownLeft v-if="index === highlighted" class="mt-0.5 size-3.5 shrink-0 text-muted-foreground" />
      </li>

      <li
        v-if="newTitle"
        :data-row="results.length"
        :class="
          cn(
            'flex cursor-pointer items-center gap-2.5 rounded-md px-2.5 py-2 text-[13px]',
            highlighted === results.length && 'bg-accent',
          )
        "
        @mousemove="highlighted = results.length"
        @click="create"
      >
        <Plus class="size-3.5 shrink-0 text-muted-foreground" />
        <span class="truncate">{{ t('common.newNoteNamed', { title: newTitle }) }}</span>
        <span class="ml-auto shrink-0 text-[11px] text-muted-foreground">Ctrl+Enter</span>
      </li>

      <li v-if="rowCount === 0" class="py-10 text-center text-xs text-muted-foreground">
        {{ query.trim() ? t('common.noMatches') : t('search.noNotes') }}
      </li>
    </ul>

    <footer class="flex gap-3 border-t px-3.5 py-1.5 text-[11px] text-muted-foreground">
      <span>{{ t('search.navigate') }}</span>
      <span>{{ t('search.open') }}</span>
      <span>{{ t('search.close') }}</span>
      <span v-if="!query.trim()" class="ml-auto">{{ t('search.recent') }}</span>
    </footer>
  </div>
</template>
