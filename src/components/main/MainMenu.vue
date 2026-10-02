<script setup lang="ts">
import { Check, Download, EyeOff, FolderOpen, ImageOff, Keyboard, Layers, Settings, Upload } from 'lucide-vue-next'

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { buttonVariants } from '@/components/ui/button'

defineProps<{ autostart: boolean; pinnedTranslucent: boolean }>()

const emit = defineEmits<{
  'show-all': []
  'toggle-hidden': []
  'toggle-autostart': []
  'toggle-translucent': []
  shortcuts: []
  'cleanup-images': []
  export: []
  import: []
  'open-backups': []
}>()

/** Checkbox items keep the menu open so the new state is visible. */
function keepOpen(event: Event, action: 'toggle-autostart' | 'toggle-translucent') {
  event.preventDefault()
  if (action === 'toggle-autostart') emit('toggle-autostart')
  else emit('toggle-translucent')
}
</script>

<template>
  <DropdownMenu :modal="false">
    <DropdownMenuTrigger :class="buttonVariants({ variant: 'ghost', size: 'icon-sm' })" title="設定">
      <Settings class="size-4" />
    </DropdownMenuTrigger>

    <DropdownMenuContent class="w-60">
      <DropdownMenuItem @select="emit('show-all')">
        <Layers />
        顯示全部便籤
      </DropdownMenuItem>
      <DropdownMenuItem @select="emit('toggle-hidden')">
        <EyeOff />
        隱藏 / 顯示便籤
        <span class="ml-auto text-[11px] text-muted-foreground">Ctrl+Alt+H</span>
      </DropdownMenuItem>
      <DropdownMenuSeparator />
      <DropdownMenuItem @select="keepOpen($event, 'toggle-translucent')">
        <span class="flex size-3.5 items-center justify-center rounded-[3px] border border-input">
          <Check v-if="pinnedTranslucent" class="size-3! text-foreground!" />
        </span>
        置頂便籤未使用時半透明
      </DropdownMenuItem>
      <DropdownMenuItem @select="keepOpen($event, 'toggle-autostart')">
        <span class="flex size-3.5 items-center justify-center rounded-[3px] border border-input">
          <Check v-if="autostart" class="size-3! text-foreground!" />
        </span>
        開機時自動啟動
      </DropdownMenuItem>
      <DropdownMenuItem @select="emit('shortcuts')">
        <Keyboard />
        自訂快捷鍵…
      </DropdownMenuItem>
      <DropdownMenuSeparator />
      <DropdownMenuItem @select="emit('export')">
        <Download />
        匯出便籤 (JSON)
      </DropdownMenuItem>
      <DropdownMenuItem @select="emit('import')">
        <Upload />
        匯入便籤 (JSON)
      </DropdownMenuItem>
      <DropdownMenuItem @select="emit('open-backups')">
        <FolderOpen />
        開啟自動備份資料夾
      </DropdownMenuItem>
      <DropdownMenuItem @select="emit('cleanup-images')">
        <ImageOff />
        清理未使用的圖片
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
