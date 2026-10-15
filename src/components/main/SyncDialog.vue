<script setup lang="ts">
import { computed, watch } from 'vue'
import { CircleAlert, FolderSync, RefreshCw, TriangleAlert } from 'lucide-vue-next'

import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'
import { t } from '@/i18n'
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
    r.uploaded && t('sync.uploaded', { count: r.uploaded }),
    r.downloaded && t('sync.downloaded', { count: r.downloaded }),
    r.deleted && t('sync.deleted', { count: r.deleted }),
    r.images && t('sync.images', { count: r.images }),
  ].filter(Boolean)
  return parts.length ? parts.join(t('sync.listSeparator')) : t('sync.noChanges')
})
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="max-w-md">
      <DialogTitle class="flex items-center gap-1.5 text-[15px] font-semibold">
        <FolderSync class="size-4" />
        {{ t('sync.title') }}
      </DialogTitle>
      <DialogDescription class="mt-1 text-xs leading-relaxed text-muted-foreground">
        {{ t('sync.description') }}
      </DialogDescription>

      <div v-if="!sync.enabled" class="mt-4">
        <Button class="w-full" @click="sync.chooseFolder()">
          <FolderSync class="size-3.5" />
          {{ t('sync.chooseFolder') }}
        </Button>
        <p class="mt-2 text-[11px] text-muted-foreground">
          {{ t('sync.subfolderHint') }}
        </p>
      </div>

      <div v-else class="mt-4 space-y-3 text-xs">
        <div>
          <div class="mb-1 text-[11px] text-muted-foreground">{{ t('sync.folder') }}</div>
          <div class="rounded-md border bg-muted/40 px-2 py-1.5 break-all">{{ status.folder }}</div>
          <p v-if="!looksLikeCloudFolder(status.folder)" class="mt-1.5 flex gap-1 text-[11px] text-amber-700">
            <TriangleAlert class="mt-px size-3 shrink-0" />
            {{ t('sync.notCloudFolder') }}
          </p>
        </div>

        <!-- The folder used before is gone: sync is paused until the user picks what to do. -->
        <div v-if="status.folderMissing" class="space-y-2 rounded-md border border-amber-300 bg-amber-50 px-2.5 py-2">
          <p class="flex gap-1.5 font-medium text-amber-800">
            <TriangleAlert class="mt-px size-3.5 shrink-0" />
            {{ t('sync.folderMissing') }}
          </p>
          <p class="leading-relaxed text-amber-900/80">
            {{ t('sync.folderMissingDetail') }}
          </p>
          <div class="flex flex-col gap-1.5">
            <Button size="sm" @click="sync.recreateFolder()">{{ t('sync.recreate') }}</Button>
            <Button size="sm" variant="outline" @click="sync.chooseFolder()">{{ t('sync.chooseOther') }}</Button>
            <Button size="sm" variant="ghost-destructive" @click="sync.disable()">{{ t('sync.stop') }}</Button>
          </div>
        </div>

        <div v-else class="rounded-md border px-2.5 py-2">
          <div v-if="status.running" class="flex items-center gap-1.5">
            <RefreshCw class="size-3.5 animate-spin" />
            {{ t('sync.syncing') }}
          </div>
          <div v-else-if="status.lastError" class="flex gap-1.5 text-destructive">
            <CircleAlert class="mt-px size-3.5 shrink-0" />
            <span class="break-all">{{ status.lastError }}</span>
          </div>
          <template v-else-if="status.lastSyncAt">
            <div>{{ t('sync.lastSync', { time: formatNoteTime(status.lastSyncAt), summary }) }}</div>
            <div v-if="report?.conflicts" class="mt-1 text-amber-700">
              {{ t('sync.conflicts', { count: report.conflicts }) }}
            </div>
          </template>
          <div v-else class="text-muted-foreground">{{ t('sync.neverSynced') }}</div>

          <details v-if="report?.warnings.length" class="mt-1.5 text-[11px] text-muted-foreground">
            <summary class="cursor-pointer">{{ t('sync.warnings', { count: report.warnings.length }) }}</summary>
            <ul class="mt-1 max-h-24 list-disc space-y-0.5 overflow-y-auto pl-4 break-all">
              <li v-for="(warning, index) in report.warnings" :key="index">{{ warning }}</li>
            </ul>
          </details>
        </div>

        <div v-if="!status.folderMissing" class="flex flex-wrap items-center gap-1.5">
          <Button size="sm" :disabled="status.running" @click="sync.syncNow()">
            <RefreshCw :class="cn('size-3.5', status.running && 'animate-spin')" />
            {{ t('sync.syncNow') }}
          </Button>
          <Button size="sm" variant="outline" @click="sync.chooseFolder()">{{ t('sync.changeFolder') }}</Button>
          <Button size="sm" variant="ghost-destructive" class="ml-auto" @click="sync.disable()">
            {{ t('sync.stop') }}
          </Button>
        </div>
      </div>

      <p v-if="sync.error" class="mt-2 text-xs text-destructive">{{ sync.error }}</p>
    </DialogContent>
  </Dialog>
</template>
