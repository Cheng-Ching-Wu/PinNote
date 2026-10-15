<script setup lang="ts">
import { watch } from 'vue'
import { CircleAlert, CircleArrowUp, CircleCheck, Download, ExternalLink, RefreshCw, StickyNote } from 'lucide-vue-next'

import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'
import { Switch } from '@/components/ui/switch'
import { APP_LINKS } from '@/constants/app'
import { type MessageKey, t } from '@/i18n'
import { cn } from '@/lib/utils'
import { settingsService } from '@/services/settingsService'
import { useSettingsStore } from '@/stores/settings'
import { useUpdateStore } from '@/stores/update'
import { formatBytes } from '@/utils/format'

/** Settings → 應用程式說明: version, links and everything about updates. */
const update = useUpdateStore()
const settings = useSettingsStore()

watch(
  () => update.dialogOpen,
  (open) => {
    if (open) void update.load()
  },
)

const LINKS: { key: MessageKey; url: string }[] = [
  { key: 'about.source', url: APP_LINKS.source },
  { key: 'about.releases', url: APP_LINKS.releases },
  { key: 'about.issues', url: APP_LINKS.issues },
  { key: 'about.license', url: APP_LINKS.license },
]

function openLink(url: string) {
  void settingsService.openExternal(url).catch(console.error)
}

function setAutoCheck(enabled: boolean) {
  void settings.updateSettings({ autoCheckUpdates: enabled })
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
      <div class="flex items-center gap-3">
        <span class="flex size-10 shrink-0 items-center justify-center rounded-lg bg-[#fff3b0] text-[#8a6d00]">
          <StickyNote class="size-5" />
        </span>
        <div class="min-w-0">
          <DialogTitle class="text-[15px] font-semibold">PinNote</DialogTitle>
          <p class="text-xs text-muted-foreground tabular-nums">
            {{ update.currentVersion ? t('about.version', { version: update.currentVersion }) : '' }}
          </p>
        </div>
      </div>
      <DialogDescription class="mt-3 text-xs leading-relaxed text-muted-foreground">
        {{ t('about.description') }}
      </DialogDescription>

      <div class="mt-2 flex flex-wrap gap-x-3 gap-y-1 text-xs">
        <button
          v-for="link in LINKS"
          :key="link.key"
          type="button"
          class="inline-flex items-center gap-1 text-foreground/80 underline-offset-2 hover:underline"
          @click="openLink(link.url)"
        >
          {{ t(link.key) }}
          <ExternalLink class="size-3" />
        </button>
      </div>

      <!-- Updates -->
      <section class="mt-4 space-y-2.5 rounded-md border px-3 py-2.5 text-xs">
        <div class="flex items-center gap-2">
          <h3 class="flex-1 text-[13px] font-medium">{{ t('update.title') }}</h3>
          <Button
            v-if="!update.installing"
            size="sm"
            variant="outline"
            class="h-7"
            :disabled="update.checking"
            @click="update.check()"
          >
            <RefreshCw :class="cn('size-3.5', update.checking && 'animate-spin')" />
            {{ update.checking ? t('update.checking') : t('update.check') }}
          </Button>
        </div>

        <template v-if="update.available && !update.checking">
          <p class="flex items-center gap-1.5 font-medium text-sky-800">
            <CircleArrowUp class="size-3.5 shrink-0" />
            {{ t('update.newVersion', { version: update.available.version }) }}
          </p>
          <div
            v-if="update.available.notes"
            class="max-h-40 overflow-y-auto rounded-md bg-muted/50 px-2.5 py-2 leading-relaxed break-words whitespace-pre-wrap"
          >
            {{ update.available.notes }}
          </div>

          <div v-if="update.installing" class="space-y-1.5">
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
          <div v-else class="flex justify-end">
            <Button size="sm" class="h-7" @click="update.install()">
              <Download class="size-3.5" />
              {{ t('update.install') }}
            </Button>
          </div>
        </template>

        <p v-else-if="update.upToDate && !update.checking" class="flex items-center gap-1.5">
          <CircleCheck class="size-3.5 text-emerald-600" />
          {{ t('update.upToDate') }}
        </p>

        <p v-if="update.error" class="flex gap-1.5 break-all text-destructive">
          <CircleAlert class="mt-px size-3.5 shrink-0" />
          {{ update.error }}
        </p>

        <label class="flex cursor-pointer items-center gap-2 border-t pt-2.5 select-none">
          <Switch :model-value="settings.settings.autoCheckUpdates" @update:model-value="setAutoCheck" />
          <span class="flex-1">{{ t('update.autoCheck') }}</span>
        </label>
        <p class="-mt-1.5 text-[11px] text-muted-foreground">{{ t('update.autoCheckHint') }}</p>
      </section>
    </DialogContent>
  </Dialog>
</template>
