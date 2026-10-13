<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import {
  CircleAlert,
  CircleCheck,
  Cloud,
  CloudAlert,
  MonitorUp,
  Plus,
  RefreshCw,
  Search,
  StickyNote,
  X,
} from 'lucide-vue-next'

import BatchActionBar from '@/components/main/BatchActionBar.vue'
import MainMenu from '@/components/main/MainMenu.vue'
import NoteListItem from '@/components/main/NoteListItem.vue'
import NoteSortMenu from '@/components/main/NoteSortMenu.vue'
import ShortcutSettingsDialog from '@/components/main/ShortcutSettingsDialog.vue'
import SyncDialog from '@/components/main/SyncDialog.vue'
import TrashBar from '@/components/main/TrashBar.vue'
import TagEditDialog from '@/components/tags/TagEditDialog.vue'
import TagFilterBar from '@/components/tags/TagFilterBar.vue'
import TagManagerDialog from '@/components/tags/TagManagerDialog.vue'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useListReorder } from '@/composables/useListReorder'
import { useNoteSelection } from '@/composables/useNoteSelection'
import type { Note, NoteStatus } from '@/models/note'
import { matchesQuery, useNoteStore } from '@/stores/note'
import { useSettingsStore } from '@/stores/settings'
import { useShortcutStore } from '@/stores/shortcuts'
import { useSyncStore } from '@/stores/sync'
import { formatNoteTime } from '@/utils/time'
import { formatAccelerator } from '@/utils/accelerator'
import { isNoteSort, mergeVisibleOrder, type NoteSort, sortNotes } from '@/utils/sort'
import { isStringArray, loadStored, saveStored } from '@/utils/storage'
import { collectTags, matchesTagFilter, UNTAGGED_KEY } from '@/utils/tags'

type Tab = NoteStatus

const store = useNoteStore()
const settings = useSettingsStore()

const sync = useSyncStore()
const syncDialogOpen = ref(false)

const syncTitle = computed(() => {
  const status = sync.status
  if (status.running) return '同步中…'
  if (status.folderMissing) return '同步資料夾不見了，已暫停同步'
  if (status.lastError) return `同步失敗：${status.lastError}`
  return status.lastSyncAt ? `已同步（${formatNoteTime(status.lastSyncAt)}）` : '等待同步'
})

const errorText = computed(() => store.lastError ?? settings.error)

function dismissError() {
  store.lastError = null
  settings.error = null
}

const tab = ref<Tab>('active')
const query = ref('')
const shortcutStore = useShortcutStore()
const shortcutDialogOpen = ref(false)
const failedShortcuts = computed(() => shortcutStore.failed)
const shortcutHint = computed(() => {
  const active = shortcutStore.statuses.filter((s) => s.accelerator)
  return active.length
    ? active.map((s) => `${formatAccelerator(s.accelerator)} ${s.label}`).join(' · ')
    : '關閉此視窗會縮到系統匣'
})

const tabs = computed<{ value: Tab; label: string; notes: Note[] }[]>(() => [
  { value: 'active', label: '全部', notes: store.activeNotes },
  { value: 'archived', label: '封存', notes: store.archivedNotes },
  { value: 'deleted', label: '垃圾桶', notes: store.deletedNotes },
])

// ---- Sorting (the trash is always newest-deleted first) ----

const SORT_STORAGE_KEY = 'pinnote.sort'
const sort = ref<NoteSort>(loadStored(SORT_STORAGE_KEY, 'updated', isNoteSort))
watch(sort, (value) => saveStored(SORT_STORAGE_KEY, value))

const tabNotes = computed(() => {
  const notes = tabs.value.find((t) => t.value === tab.value)?.notes ?? []
  return tab.value === 'deleted' ? notes : sortNotes(notes, sort.value)
})

// ---- Tag filter ----

const tagEditIds = ref<string[]>([])
const tagManagerOpen = ref(false)

const TAG_FILTER_STORAGE_KEY = 'pinnote.tagFilter'
const tagFilter = ref<string[]>(loadStored(TAG_FILTER_STORAGE_KEY, [], isStringArray))
watch(tagFilter, (keys) => saveStored(TAG_FILTER_STORAGE_KEY, keys))

/** Tags of the notes in the current tab, so the counts match what the filter would show. */
const tabTags = computed(() => collectTags(tabNotes.value))
const untaggedCount = computed(() => tabNotes.value.filter((note) => note.tags.length === 0).length)

/** Selected tags that exist in this tab; tags missing here are kept for the other tabs but ignored. */
const activeTagFilter = computed(() => {
  const available = new Set(tabTags.value.map((tag) => tag.key))
  if (untaggedCount.value > 0) available.add(UNTAGGED_KEY)
  return new Set(tagFilter.value.filter((key) => available.has(key)))
})

const visibleNotes = computed(() =>
  tabNotes.value.filter((note) => matchesTagFilter(note, activeTagFilter.value) && matchesQuery(note, query.value)),
)
const visibleIds = computed(() => visibleNotes.value.map((note) => note.id))

const emptyText = computed(() => {
  if (query.value.trim() || activeTagFilter.value.size > 0) return '找不到符合的便籤'
  if (tab.value === 'archived') return '沒有封存的便籤'
  if (tab.value === 'deleted') return '垃圾桶是空的'
  return '還沒有便籤，按「新增便籤」開始'
})

// ---- Batch mode (multi-select) ----

/** Switched on in the list toolbar: clicks select notes instead of opening them. */
const batchMode = ref(false)
const selection = useNoteSelection(visibleIds)
watch(tab, () => selection.clear())
watch(batchMode, (on) => {
  if (!on) selection.clear()
})

/** In batch mode a click toggles and Shift+click selects a range; otherwise a click opens the note. */
function onActivate(note: Note, event: MouseEvent | KeyboardEvent) {
  if (!batchMode.value) {
    if (note.status === 'active') void store.openNoteWindow(note.id)
  } else if (event.shiftKey) {
    selection.selectRange(note.id)
  } else {
    selection.toggle(note.id)
  }
}

/** Runs an action on the selection; batch mode stays on for the next one. */
async function runBatch(action: (ids: string[]) => Promise<unknown>, clearAfter = true) {
  const ids = selection.ids.value
  if (ids.length === 0) return
  await action(ids)
  if (clearAfter) selection.clear()
}

// ---- Manual order (drag and drop) ----

const reorderable = computed(() => sort.value === 'manual' && tab.value !== 'deleted' && !batchMode.value)

const reorder = useListReorder(visibleIds, (ids) => {
  // A filter may hide notes: they keep their slots in the full list.
  const all = sortNotes(tabNotes.value, 'manual').map((note) => note.id)
  void store.reorderNotes(mergeVisibleOrder(all, ids))
})

function restore(note: Note) {
  return note.status === 'deleted' ? store.restoreDeletedNote(note.id) : store.restoreNote(note.id)
}

const dialogOpen = computed(
  () => shortcutDialogOpen.value || tagManagerOpen.value || syncDialogOpen.value || tagEditIds.value.length > 0,
)

function isTyping(target: EventTarget | null) {
  return target instanceof HTMLElement && (target.isContentEditable || ['INPUT', 'TEXTAREA'].includes(target.tagName))
}

function onKeydown(event: KeyboardEvent) {
  const key = event.key.toLowerCase()
  if (event.ctrlKey && !event.altKey && key === 'n') {
    event.preventDefault()
    void store.createNote()
    return
  }
  if (!batchMode.value || dialogOpen.value || isTyping(event.target)) return
  if (event.ctrlKey && !event.altKey && key === 'a') {
    event.preventDefault()
    selection.selectAll()
  } else if (key === 'escape') {
    event.preventDefault()
    batchMode.value = false
  }
}

onMounted(async () => {
  window.addEventListener('keydown', onKeydown)
  await Promise.all([store.loadNotes(), settings.loadSettings(), settings.loadAutostart(), sync.load()])
  await sync.startSync((status) => {
    const conflicts = status.lastReport?.conflicts ?? 0
    if (conflicts > 0) {
      store.notice = `有 ${conflicts} 張便籤在兩台電腦都被修改，已採用較新的內容，較舊的內容保留在「歷史版本」`
    }
  })
  await shortcutStore.load()
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
          <Button v-if="sync.enabled" variant="ghost" size="icon-sm" :title="syncTitle" @click="syncDialogOpen = true">
            <RefreshCw v-if="sync.status.running" class="size-4 animate-spin text-muted-foreground" />
            <CloudAlert v-else-if="sync.status.folderMissing" class="size-4 text-amber-600" />
            <CloudAlert v-else-if="sync.status.lastError" class="size-4 text-destructive" />
            <Cloud v-else class="size-4 text-muted-foreground" />
          </Button>
          <MainMenu
            :autostart="settings.autostart"
            :pinned-translucent="settings.settings.pinnedTranslucent"
            @show-all="store.showAllNotes()"
            @arrange="(layout, anchor) => store.arrangeNotes(layout, anchor)"
            @toggle-hidden="store.toggleNotesHidden()"
            @toggle-autostart="settings.setAutostart(!settings.autostart)"
            @shortcuts="shortcutDialogOpen = true"
            @manage-tags="tagManagerOpen = true"
            @cleanup-images="store.cleanupImages()"
            @toggle-translucent="settings.updateSettings({ pinnedTranslucent: !settings.settings.pinnedTranslucent })"
            @export="store.exportNotes()"
            @import="store.importNotes()"
            @open-backups="settings.openBackupFolder()"
            @sync="syncDialogOpen = true"
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

      <TagFilterBar v-if="tabTags.length > 0" v-model="tagFilter" :tags="tabTags" :untagged-count="untaggedCount" />
    </header>

    <div class="flex h-8 shrink-0 items-center gap-2 border-b px-3">
      <TrashBar
        v-if="tab === 'deleted'"
        :retention-days="settings.settings.trashRetentionDays"
        :count="tabNotes.length"
        class="min-w-0 flex-1"
        @update:retention-days="settings.updateSettings({ trashRetentionDays: $event })"
        @empty="store.emptyTrash()"
      />
      <template v-else>
        <span class="min-w-0 flex-1 truncate text-[11px] text-muted-foreground tabular-nums">
          {{ visibleNotes.length }} 張便籤
          <template v-if="reorderable && visibleNotes.length > 1">· 拖曳即可調整順序</template>
        </span>
        <!-- Workspace by tag: put just the filtered notes on the desktop. -->
        <Button
          v-if="tab === 'active' && activeTagFilter.size > 0 && visibleNotes.length > 0"
          variant="ghost"
          size="sm"
          class="h-6 shrink-0 gap-1 px-1.5 text-[11px] font-normal text-muted-foreground"
          title="桌面上只顯示目前篩選出的便籤，其他便籤會關閉（可用「顯示全部便籤」叫回）"
          @click="store.showOnlyNotes(visibleIds)"
        >
          <MonitorUp class="size-3" />
          桌面只顯示這些
        </Button>
        <NoteSortMenu v-model="sort" />
      </template>
      <label
        class="flex shrink-0 cursor-pointer items-center gap-1.5 text-[11px] text-muted-foreground select-none"
        title="開啟後點擊便籤即可選取（Shift+點擊選取範圍、Ctrl+A 全選），一次封存、刪除或修改；Esc 關閉"
      >
        <Switch v-model="batchMode" />
        批次操作
      </label>
    </div>

    <main class="min-h-0 flex-1 space-y-2 overflow-y-auto p-3" @dragover.prevent @drop="reorder.onDrop">
      <NoteListItem
        v-for="note in visibleNotes"
        :key="note.id"
        :note="note"
        :selecting="batchMode"
        :selected="selection.isSelected(note.id)"
        :reorderable="reorderable"
        :drop-position="reorder.dropPositionOf(note.id)"
        :draggable="reorderable"
        :class="reorder.dragging.value === note.id && 'opacity-40'"
        @dragstart="reorder.onDragStart($event, note.id)"
        @dragover="reorder.onDragOver($event, note.id)"
        @dragend="reorder.onDragEnd"
        @activate="onActivate(note, $event)"
        @open="store.openNoteWindow(note.id)"
        @edit-tags="tagEditIds = [note.id]"
        @archive="store.archiveNote(note.id)"
        @restore="restore(note)"
        @delete="store.deleteNote(note.id)"
        @delete-permanently="store.deleteNotePermanently(note.id)"
      />
      <p v-if="store.loaded && visibleNotes.length === 0" class="pt-10 text-center text-xs text-muted-foreground">
        {{ emptyText }}
      </p>
    </main>

    <!-- Stays until resolved: notes are no longer reaching other devices. -->
    <div
      v-if="sync.status.folderMissing"
      class="mx-3 mb-2 flex items-start gap-2 rounded-md border border-amber-300 bg-amber-50 px-2.5 py-2 text-xs text-amber-900"
    >
      <CloudAlert class="mt-px size-3.5 shrink-0" />
      <span class="flex-1">同步資料夾不見了，已暫停同步。這台電腦的便籤都還在。</span>
      <button type="button" class="shrink-0 font-medium hover:underline" @click="syncDialogOpen = true">處理…</button>
    </div>
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

    <BatchActionBar
      v-if="batchMode"
      :count="selection.count.value"
      :all-selected="selection.allSelected.value"
      :status="tab"
      @select-all="selection.selectAll()"
      @clear="selection.clear()"
      @exit="batchMode = false"
      @color="(color) => runBatch((ids) => store.setColorOfNotes(ids, color), false)"
      @tags="tagEditIds = [...selection.ids.value]"
      @archive="runBatch(store.archiveNotes)"
      @restore="runBatch(store.restoreNotes)"
      @delete="runBatch(store.deleteNotes)"
      @delete-permanently="runBatch(store.deleteNotesPermanently)"
    />
    <footer v-else class="border-t px-4 py-2 text-[11px] text-muted-foreground">
      <p
        v-for="item in failedShortcuts"
        :key="item.accelerator"
        class="text-destructive"
        :title="item.error ?? ''"
      >
        快捷鍵 {{ formatAccelerator(item.accelerator) }}（{{ item.label }}）註冊失敗，可能已被其他程式占用，
        <button type="button" class="underline" @click="shortcutDialogOpen = true">更換按鍵</button>
      </p>
      <p v-if="failedShortcuts.length === 0" class="truncate" :title="shortcutHint">{{ shortcutHint }}</p>
    </footer>

    <ShortcutSettingsDialog v-model:open="shortcutDialogOpen" />
    <TagEditDialog v-model:note-ids="tagEditIds" />
    <TagManagerDialog v-model:open="tagManagerOpen" />
    <SyncDialog v-model:open="syncDialogOpen" />
  </div>
</template>
