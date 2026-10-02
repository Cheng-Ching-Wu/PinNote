<script setup lang="ts">
import { Check, Download, FolderOpen, Layers, Settings, Upload } from 'lucide-vue-next'

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { buttonVariants } from '@/components/ui/button'

defineProps<{ autostart: boolean }>()

const emit = defineEmits<{
  'show-all': []
  'toggle-autostart': []
  export: []
  import: []
  'open-backups': []
}>()

/** Keep the menu open so the new checkbox state is visible. */
function toggleAutostart(event: Event) {
  event.preventDefault()
  emit('toggle-autostart')
}
</script>

<template>
  <DropdownMenu :modal="false">
    <DropdownMenuTrigger :class="buttonVariants({ variant: 'ghost', size: 'icon-sm' })" title="設定">
      <Settings class="size-4" />
    </DropdownMenuTrigger>

    <DropdownMenuContent class="w-52">
      <DropdownMenuItem @select="emit('show-all')">
        <Layers />
        顯示全部便籤
      </DropdownMenuItem>
      <DropdownMenuSeparator />
      <DropdownMenuItem @select="toggleAutostart">
        <span class="flex size-3.5 items-center justify-center rounded-[3px] border border-input">
          <Check v-if="autostart" class="size-3! text-foreground!" />
        </span>
        開機時自動啟動
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
    </DropdownMenuContent>
  </DropdownMenu>
</template>
