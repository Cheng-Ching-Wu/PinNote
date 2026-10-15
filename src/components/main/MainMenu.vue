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
  Languages,
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
  ARRANGE_ANCHORS,
  ARRANGE_LAYOUTS,
  type ArrangeAnchor,
  type ArrangeLayout,
  isArrangeAnchor,
  isArrangeLayout,
} from '@/constants/arrange'
import { LANGUAGE_SETTINGS, type LanguageSetting, LOCALE_NAMES, t } from '@/i18n'
import { cn } from '@/lib/utils'
import { loadStored, saveStored } from '@/utils/storage'

defineProps<{ autostart: boolean; pinnedTranslucent: boolean; language: LanguageSetting }>()

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
  language: [language: LanguageSetting]
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
const languageName = (language: LanguageSetting) =>
  language === 'system' ? t('language.system') : LOCALE_NAMES[language]

function keepOpen(event: Event, action: 'toggle-autostart' | 'toggle-translucent') {
  event.preventDefault()
  if (action === 'toggle-autostart') emit('toggle-autostart')
  else emit('toggle-translucent')
}
</script>

<template>
  <DropdownMenu :modal="false">
    <DropdownMenuTrigger :class="buttonVariants({ variant: 'ghost', size: 'icon-sm' })" :title="t('menu.settings')">
      <Settings class="size-4" />
    </DropdownMenuTrigger>

    <DropdownMenuContent class="w-60">
      <DropdownMenuItem @select="emit('show-all')">
        <Layers />
        {{ t('menu.showAll') }}
      </DropdownMenuItem>
      <DropdownMenuItem @select="emit('toggle-hidden')">
        <EyeOff />
        {{ t('menu.toggleHidden') }}
        <span class="ml-auto text-[11px] text-muted-foreground">Ctrl+Alt+H</span>
      </DropdownMenuItem>
      <DropdownMenuSub>
        <DropdownMenuSubTrigger>
          <LayoutGrid />
          {{ t('arrange.title') }}
        </DropdownMenuSubTrigger>
        <DropdownMenuSubContent class="w-40">
          <DropdownMenuLabel>{{ t('arrange.layout') }}</DropdownMenuLabel>
          <DropdownMenuItem v-for="layout in ARRANGE_LAYOUTS" :key="layout" @select="chooseLayout($event, layout)">
            <Check :class="cn(layout !== arrangeLayout && 'invisible')" />
            {{ t(`arrange.layouts.${layout}`) }}
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuLabel>{{ t('arrange.placeAt') }}</DropdownMenuLabel>
          <DropdownMenuItem v-for="anchor in ARRANGE_ANCHORS" :key="anchor" @select="arrange(anchor)">
            <component :is="ANCHOR_ICONS[anchor]" />
            {{ t(`arrange.anchors.${anchor}`) }}
            <span v-if="anchor === arrangeAnchor" class="ml-auto text-[10px] text-muted-foreground">{{ t('arrange.lastUsed') }}</span>
          </DropdownMenuItem>
        </DropdownMenuSubContent>
      </DropdownMenuSub>
      <DropdownMenuSeparator />
      <DropdownMenuItem @select="keepOpen($event, 'toggle-translucent')">
        <span class="flex size-3.5 items-center justify-center rounded-[3px] border border-input">
          <Check v-if="pinnedTranslucent" class="size-3! text-foreground!" />
        </span>
        {{ t('menu.translucent') }}
      </DropdownMenuItem>
      <DropdownMenuItem @select="keepOpen($event, 'toggle-autostart')">
        <span class="flex size-3.5 items-center justify-center rounded-[3px] border border-input">
          <Check v-if="autostart" class="size-3! text-foreground!" />
        </span>
        {{ t('menu.autostart') }}
      </DropdownMenuItem>
      <DropdownMenuSub>
        <DropdownMenuSubTrigger>
          <Languages />
          {{ t('language.title') }}
        </DropdownMenuSubTrigger>
        <DropdownMenuSubContent class="w-40">
          <DropdownMenuItem v-for="option in LANGUAGE_SETTINGS" :key="option" @select="emit('language', option)">
            <Check :class="cn(option !== language && 'invisible')" />
            {{ languageName(option) }}
          </DropdownMenuItem>
        </DropdownMenuSubContent>
      </DropdownMenuSub>
      <DropdownMenuItem @select="emit('shortcuts')">
        <Keyboard />
        {{ t('menu.shortcuts') }}
      </DropdownMenuItem>
      <DropdownMenuItem @select="emit('manage-tags')">
        <Tags />
        {{ t('menu.manageTags') }}
      </DropdownMenuItem>
      <DropdownMenuSeparator />
      <DropdownMenuItem @select="emit('sync')">
        <FolderSync />
        {{ t('menu.sync') }}
      </DropdownMenuItem>
      <DropdownMenuItem @select="emit('export')">
        <Download />
        {{ t('menu.export') }}
      </DropdownMenuItem>
      <DropdownMenuItem @select="emit('import')">
        <Upload />
        {{ t('menu.import') }}
      </DropdownMenuItem>
      <DropdownMenuItem @select="emit('open-backups')">
        <FolderOpen />
        {{ t('menu.openBackups') }}
      </DropdownMenuItem>
      <DropdownMenuItem @select="emit('cleanup-images')">
        <ImageOff />
        {{ t('menu.cleanupImages') }}
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
