<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { CircleAlert, CircleCheck, Plus, Search, StickyNote, X } from 'lucide-vue-next'

import MainMenu from '@/components/main/MainMenu.vue'
import NoteListItem from '@/components/main/NoteListItem.vue'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import type { Note } from '@/models/note'
import { shortcutService, type ShortcutStatus } from '@/services/shortcutService'
import { matchesQuery, useNoteStore } from '@/stores/note'
import { useSettingsStore } from '@/stores/settings'

type Tab = 'active' | 'archived' | 'deleted'

const store = useNoteStore()
const settings = useSettingsStore()

const errorText = computed(() => store.lastError ?? settings.error)

function dismissError() {
  store.lastError = null
  settings.error = null
}

const tab = ref<Tab>('active')
const query = ref('')
const shortcut = ref<ShortcutStatus | null>(null)

const tabs = computed<{ value: Tab; label: string; notes: Note[] }[]>(() => [
  { value: 'active', label: '全部', notes: store.activeNotes },
  { value: 'archived', label: '封存', notes: store.archivedNotes },
  { value: 'deleted', label: '垃圾桶', notes: store.deletedNotes },
])

const visibleNotes = computed(() => {
  const current = tabs.value.find((t) => t.value === tab.value)?.notes ?? []
  return current.filter((note) => matchesQuery(note, query.value))
})

const emptyText = computed(() => {
  if (query.value.trim()) return '找不到符合的便籤'
  if (tab.value === 'archived') return '沒有封存的便籤'
  if (tab.value === 'deleted') return '垃圾桶是空的'
  return '還沒有便籤，按「新增便籤」開始'
})

function restore(note: Note) {
  return note.status === 'deleted' ? store.restoreDeletedNote(note.id) : store.restoreNote(note.id)
}

function onKeydown(event: KeyboardEvent) {
  if (event.ctrlKey && !event.altKey && event.key.toLowerCase() === 'n') {
    event.preventDefault()
    void store.createNote()
  }
}

onMounted(async () => {
  window.addEventListener('keydown', onKeydown)
  await Promise.all([store.loadNotes(), settings.load()])
  shortcut.value = await shortcutService.getStatus().catch(() => null)
})

onBeforeUnmount(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <div class="flex h-full flex-col">
    <header class="space-y-3 border-b bg-card px-4 pt-4 pb-3">
      <div class="flex items-center justify-between">
        <h1 class="flex items-center gap-1.5 text-[15px] font-semibold tracking-tight">
          <StickyNote class="size-4" />
          PinNote
        </h1>
        <div class="flex items-center gap-1">
          <MainMenu
            :autostart="settings.autostart"
            @show-all="store.showAllNotes()"
            @toggle-autostart="settings.setAutostart(!settings.autostart)"
            @export="store.exportNotes()"
            @import="store.importNotes()"
            @open-backups="settings.openBackupFolder()"
          />
          <Button size="sm" @click="store.createNote()">
            <Plus class="size-3.5" />
            新增便籤
          </Button>
        </div>
      </div>

      <div class="relative">
        <Search class="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
        <Input v-model="query" placeholder="搜尋便籤" class="pr-8 pl-8" />
        <button
          v-if="query"
          type="button"
          title="清除"
          class="absolute top-1/2 right-1.5 inline-flex size-5 -translate-y-1/2 items-center justify-center rounded-sm text-muted-foreground hover:bg-accent"
          @click="query = ''"
        >
          <X class="size-3.5" />
        </button>
      </div>

      <Tabs v-model="tab">
        <TabsList class="w-full">
          <TabsTrigger v-for="t in tabs" :key="t.value" :value="t.value">
            {{ t.label }}
            <span class="text-[11px] font-normal text-muted-foreground tabular-nums">{{ t.notes.length }}</span>
          </TabsTrigger>
        </TabsList>
      </Tabs>
    </header>

    <main class="min-h-0 flex-1 space-y-2 overflow-y-auto p-3">
      <NoteListItem
        v-for="note in visibleNotes"
        :key="note.id"
        :note="note"
        @open="store.openNoteWindow(note.id)"
        @archive="store.archiveNote(note.id)"
        @restore="restore(note)"
        @delete="store.deleteNote(note.id)"
        @delete-permanently="store.deleteNotePermanently(note.id)"
      />
      <p v-if="store.loaded && visibleNotes.length === 0" class="pt-10 text-center text-xs text-muted-foreground">
        {{ emptyText }}
      </p>
    </main>

    <div
      v-if="errorText"
      class="mx-3 mb-2 flex items-start gap-2 rounded-md border border-destructive/30 bg-destructive/5 px-2.5 py-2 text-xs text-destructive"
    >
      <CircleAlert class="mt-px size-3.5 shrink-0" />
      <span class="flex-1 break-all">{{ errorText }}</span>
      <button type="button" class="shrink-0 hover:underline" @click="dismissError">關閉</button>
    </div>
    <div
      v-else-if="store.notice"
      class="mx-3 mb-2 flex items-start gap-2 rounded-md border bg-card px-2.5 py-2 text-xs text-foreground"
    >
      <CircleCheck class="mt-px size-3.5 shrink-0 text-emerald-600" />
      <span class="flex-1 break-all">{{ store.notice }}</span>
      <button type="button" class="shrink-0 text-muted-foreground hover:underline" @click="store.notice = null">
        關閉
      </button>
    </div>

    <footer class="border-t px-4 py-2 text-[11px] text-muted-foreground">
      <span v-if="shortcut && !shortcut.registered" class="text-destructive" :title="shortcut.error ?? ''">
        快捷鍵 {{ shortcut.accelerator }} 註冊失敗，可能已被其他程式占用
      </span>
      <span v-else>{{ shortcut?.accelerator || 'Ctrl+Alt+N' }} 隨時新增便籤 · 關閉此視窗會縮到系統匣</span>
    </footer>
  </div>
</template>
