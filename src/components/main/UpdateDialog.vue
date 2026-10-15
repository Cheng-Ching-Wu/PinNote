<script setup lang="ts">
import { CircleAlert, CircleCheck, Download, RefreshCw } from 'lucide-vue-next'

import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'
import { t } from '@/i18n'
import { cn } from '@/lib/utils'
import { useUpdateStore } from '@/stores/update'
import { formatBytes } from '@/utils/format'

const update = useUpdateStore()

function later() {
  update.dismiss()
  update.dialogOpen = false
}
</script>

<template>
  <Dialog v-model:open="update.dialogOpen">
    <!-- While installing the dialog can't be closed: the app is about to restart. -->
    <DialogContent
      class="max-w-md"
      @escape-key-down="update.installing && $event.preventDefault()"
      @pointer-down-outside="update.installing && $event.preventDefault()"
    >
      <DialogTitle class="flex items-center gap-1.5 text-[15px] font-semibold">
        <Download class="size-4" />
        {{ t('update.title') }}
      </DialogTitle>
      <DialogDescription class="mt-1 text-xs text-muted-foreground">
        {{ t('update.currentVersion', { version: update.currentVersion }) }}
      </DialogDescription>

      <div class="mt-4 text-xs">
        <div v-if="update.checking" class="flex items-center gap-1.5">
          <RefreshCw class="size-3.5 animate-spin" />
          {{ t('update.checking') }}
        </div>

        <template v-else-if="update.available">
          <p class="text-[13px] font-medium">{{ t('update.newVersion', { version: update.available.version }) }}</p>
          <div
            v-if="update.available.notes"
            class="mt-2 max-h-48 overflow-y-auto rounded-md border bg-muted/40 px-2.5 py-2 leading-relaxed break-words whitespace-pre-wrap"
          >
            {{ update.available.notes }}
          </div>

          <div v-if="update.installing" class="mt-3 space-y-1.5">
            <div class="h-1.5 overflow-hidden rounded-full bg-muted">
              <div
                :class="cn('h-full rounded-full bg-primary transition-[width]', update.percent === null && 'w-1/3 animate-pulse')"
                :style="update.percent !== null ? { width: `${update.percent}%` } : undefined"
              />
            </div>
            <p class="text-[11px] text-muted-foreground tabular-nums">
              {{
                update.progress
                  ? t('update.downloading', {
                      downloaded: formatBytes(update.progress.downloaded),
                      total: update.progress.total ? formatBytes(update.progress.total) : '?',
                    })
                  : t('update.preparing')
              }}
            </p>
            <p class="text-[11px] text-muted-foreground">{{ t('update.restartHint') }}</p>
          </div>

          <div v-else class="mt-4 flex justify-end gap-1.5">
            <Button size="sm" variant="ghost" @click="later">{{ t('update.later') }}</Button>
            <Button size="sm" @click="update.install()">
              <Download class="size-3.5" />
              {{ t('update.install') }}
            </Button>
          </div>
        </template>

        <div v-else-if="update.upToDate" class="flex items-center gap-1.5">
          <CircleCheck class="size-3.5 text-emerald-600" />
          {{ t('update.upToDate') }}
        </div>

        <p v-if="update.error" class="mt-3 flex gap-1.5 break-all text-destructive">
          <CircleAlert class="mt-px size-3.5 shrink-0" />
          {{ update.error }}
        </p>
      </div>
    </DialogContent>
  </Dialog>
</template>
