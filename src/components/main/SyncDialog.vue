<script setup lang="ts">
import { computed, watch } from 'vue'
import { CircleAlert, FolderSync, RefreshCw, TriangleAlert } from 'lucide-vue-next'

import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'
import { cn } from '@/lib/utils'
import { looksLikeCloudFolder } from '@/services/syncService'
import { useSyncStore } from '@/stores/sync'
import { formatNoteTime } from '@/utils/time'

const open = defineModel<boolean>('open', { required: true })

const sync = useSyncStore()
const status = computed(() => sync.status)

watch(open, (isOpen) => {
  if (isOpen) void sync.load()
})

const report = computed(() => status.value.lastReport)
const summary = computed(() => {
  const r = report.value
  if (!r) return ''
  const parts = [
    r.uploaded && `上傳 ${r.uploaded}`,
    r.downloaded && `下載 ${r.downloaded}`,
    r.deleted && `刪除 ${r.deleted}`,
    r.images && `圖片 ${r.images}`,
  ].filter(Boolean)
  return parts.length ? parts.join('、') : '沒有變動'
})
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="max-w-md">
      <DialogTitle class="flex items-center gap-1.5 text-[15px] font-semibold">
        <FolderSync class="size-4" />
        跨裝置同步
      </DialogTitle>
      <DialogDescription class="mt-1 text-xs leading-relaxed text-muted-foreground">
        把便籤同步到雲端硬碟裡的資料夾（Google 雲端硬碟、OneDrive…），在每台電腦選同一個資料夾即可。
        標題、內容、顏色、標籤、封存與標籤顏色會同步；視窗位置、大小、置頂、收合與快捷鍵每台電腦各自保留。
      </DialogDescription>

      <div v-if="!sync.enabled" class="mt-4">
        <Button class="w-full" @click="sync.chooseFolder()">
          <FolderSync class="size-3.5" />
          選擇同步資料夾…
        </Button>
        <p class="mt-2 text-[11px] text-muted-foreground">
          資料會放在所選資料夾中的「PinNote Sync」子資料夾。
        </p>
      </div>

      <div v-else class="mt-4 space-y-3 text-xs">
        <div>
          <div class="mb-1 text-[11px] text-muted-foreground">同步資料夾</div>
          <div class="rounded-md border bg-muted/40 px-2 py-1.5 break-all">{{ status.folder }}</div>
          <p v-if="!looksLikeCloudFolder(status.folder)" class="mt-1.5 flex gap-1 text-[11px] text-amber-700">
            <TriangleAlert class="mt-px size-3 shrink-0" />
            這個資料夾看起來不在雲端硬碟裡，可能不會同步到其他電腦。
          </p>
        </div>

        <!-- The folder used before is gone: sync is paused until the user picks what to do. -->
        <div v-if="status.folderMissing" class="space-y-2 rounded-md border border-amber-300 bg-amber-50 px-2.5 py-2">
          <p class="flex gap-1.5 font-medium text-amber-800">
            <TriangleAlert class="mt-px size-3.5 shrink-0" />
            同步資料夾不見了，已暫停同步
          </p>
          <p class="leading-relaxed text-amber-900/80">
            以前同步用的資料夾被刪除、移動，或雲端硬碟還沒準備好。這台電腦的便籤都還在。
            如果資料夾之後回來，會自動繼續同步。
          </p>
          <div class="flex flex-col gap-1.5">
            <Button size="sm" @click="sync.recreateFolder()">重新建立資料夾並上傳這台電腦的便籤</Button>
            <Button size="sm" variant="outline" @click="sync.chooseFolder()">選擇其他資料夾…</Button>
            <Button size="sm" variant="ghost-destructive" @click="sync.disable()">停止同步</Button>
          </div>
        </div>

        <div v-else class="rounded-md border px-2.5 py-2">
          <div v-if="status.running" class="flex items-center gap-1.5">
            <RefreshCw class="size-3.5 animate-spin" />
            同步中…
          </div>
          <div v-else-if="status.lastError" class="flex gap-1.5 text-destructive">
            <CircleAlert class="mt-px size-3.5 shrink-0" />
            <span class="break-all">{{ status.lastError }}</span>
          </div>
          <template v-else-if="status.lastSyncAt">
            <div>上次同步：{{ formatNoteTime(status.lastSyncAt) }}（{{ summary }}）</div>
            <div v-if="report?.conflicts" class="mt-1 text-amber-700">
              {{ report.conflicts }} 張便籤在兩台電腦都被修改，已採用較新的內容，較舊的內容保留在該便籤的「歷史版本」。
            </div>
          </template>
          <div v-else class="text-muted-foreground">尚未同步</div>

          <details v-if="report?.warnings.length" class="mt-1.5 text-[11px] text-muted-foreground">
            <summary class="cursor-pointer">{{ report.warnings.length }} 個檔案有問題</summary>
            <ul class="mt-1 max-h-24 list-disc space-y-0.5 overflow-y-auto pl-4 break-all">
              <li v-for="(warning, index) in report.warnings" :key="index">{{ warning }}</li>
            </ul>
          </details>
        </div>

        <div v-if="!status.folderMissing" class="flex flex-wrap items-center gap-1.5">
          <Button size="sm" :disabled="status.running" @click="sync.syncNow()">
            <RefreshCw :class="cn('size-3.5', status.running && 'animate-spin')" />
            立即同步
          </Button>
          <Button size="sm" variant="outline" @click="sync.chooseFolder()">變更資料夾…</Button>
          <Button size="sm" variant="ghost-destructive" class="ml-auto" @click="sync.disable()">停止同步</Button>
        </div>
      </div>

      <p v-if="sync.error" class="mt-2 text-xs text-destructive">{{ sync.error }}</p>
    </DialogContent>
  </Dialog>
</template>
