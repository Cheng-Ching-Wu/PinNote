<script setup lang="ts">
import { type Component, ref, watch } from 'vue'
import {
  ArrowDownLeft,
  ArrowDownRight,
  ArrowUpLeft,
  ArrowUpRight,
  Check,
  Crosshair,
  Download,
  EyeOff,
  FolderOpen,
  FolderSync,
  ImageOff,
  Keyboard,
  LayoutGrid,
  Layers,
  Settings,
  Tags,
  Upload,
} from 'lucide-vue-next'

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { buttonVariants } from '@/components/ui/button'
import {
  ARRANGE_ANCHOR_LABELS,
  ARRANGE_ANCHORS,
  ARRANGE_LAYOUT_LABELS,
  ARRANGE_LAYOUTS,
  type ArrangeAnchor,
  type ArrangeLayout,
  isArrangeAnchor,
  isArrangeLayout,
} from '@/constants/arrange'
import { cn } from '@/lib/utils'
import { loadStored, saveStored } from '@/utils/storage'

defineProps<{ autostart: boolean; pinnedTranslucent: boolean }>()

const emit = defineEmits<{
  'show-all': []
  arrange: [layout: ArrangeLayout, anchor: ArrangeAnchor]
  'toggle-hidden': []
  'toggle-autostart': []
  'toggle-translucent': []
  shortcuts: []
  'manage-tags': []
  'cleanup-images': []
  export: []
  import: []
  'open-backups': []
  sync: []
}>()

// ---- 整理桌面便籤 (submenu): pick a layout, then a corner to run it ----

const LAYOUT_STORAGE_KEY = 'pinnote.arrangeLayout'
const ANCHOR_STORAGE_KEY = 'pinnote.arrangeAnchor'
const arrangeLayout = ref<ArrangeLayout>(loadStored(LAYOUT_STORAGE_KEY, 'grid', isArrangeLayout))
/** Last used position, marked in the menu. */
const arrangeAnchor = ref<ArrangeAnchor>(loadStored(ANCHOR_STORAGE_KEY, 'top-left', isArrangeAnchor))
watch(arrangeLayout, (value) => saveStored(LAYOUT_STORAGE_KEY, value))
watch(arrangeAnchor, (value) => saveStored(ANCHOR_STORAGE_KEY, value))

const ANCHOR_ICONS: Record<ArrangeAnchor, Component> = {
  'top-left': ArrowUpLeft,
  'top-right': ArrowUpRight,
  'bottom-left': ArrowDownLeft,
  'bottom-right': ArrowDownRight,
  center: Crosshair,
}

/** Choosing the layout keeps the submenu open so a position can be picked next. */
function chooseLayout(event: Event, layout: ArrangeLayout) {
  event.preventDefault()
  arrangeLayout.value = layout
}

function arrange(anchor: ArrangeAnchor) {
  arrangeAnchor.value = anchor
  emit('arrange', arrangeLayout.value, anchor)
}

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
      <DropdownMenuSub>
        <DropdownMenuSubTrigger>
          <LayoutGrid />
          整理桌面便籤
        </DropdownMenuSubTrigger>
        <DropdownMenuSubContent class="w-40">
          <DropdownMenuLabel>排列方式</DropdownMenuLabel>
          <DropdownMenuItem v-for="layout in ARRANGE_LAYOUTS" :key="layout" @select="chooseLayout($event, layout)">
            <Check :class="cn(layout !== arrangeLayout && 'invisible')" />
            {{ ARRANGE_LAYOUT_LABELS[layout] }}
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuLabel>放在</DropdownMenuLabel>
          <DropdownMenuItem v-for="anchor in ARRANGE_ANCHORS" :key="anchor" @select="arrange(anchor)">
            <component :is="ANCHOR_ICONS[anchor]" />
            {{ ARRANGE_ANCHOR_LABELS[anchor] }}
            <span v-if="anchor === arrangeAnchor" class="ml-auto text-[10px] text-muted-foreground">上次</span>
          </DropdownMenuItem>
        </DropdownMenuSubContent>
      </DropdownMenuSub>
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
      <DropdownMenuItem @select="emit('manage-tags')">
        <Tags />
        管理標籤…
      </DropdownMenuItem>
      <DropdownMenuSeparator />
      <DropdownMenuItem @select="emit('sync')">
        <FolderSync />
        跨裝置同步…
      </DropdownMenuItem>
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
