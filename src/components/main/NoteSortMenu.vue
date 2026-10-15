<script setup lang="ts">
import { ArrowDownUp, Check } from 'lucide-vue-next'

import { buttonVariants } from '@/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { t } from '@/i18n'
import { cn } from '@/lib/utils'
import { NOTE_SORTS } from '@/utils/sort'
import type { NoteSort } from '@/utils/sort'

const sort = defineModel<NoteSort>({ required: true })
</script>

<template>
  <DropdownMenu :modal="false">
    <DropdownMenuTrigger
      :class="cn(buttonVariants({ variant: 'ghost', size: 'sm' }), 'h-6 gap-1 px-1.5 text-[11px] font-normal text-muted-foreground')"
      :title="t('sort.title')"
    >
      <ArrowDownUp class="size-3" />
      {{ t(`sort.options.${sort}`) }}
    </DropdownMenuTrigger>
    <DropdownMenuContent class="w-36">
      <DropdownMenuItem v-for="option in NOTE_SORTS" :key="option" @select="sort = option">
        <Check :class="cn(option !== sort && 'invisible')" />
        {{ t(`sort.options.${option}`) }}
        <span v-if="option === 'manual'" class="ml-auto text-[10px] text-muted-foreground">{{ t('sort.drag') }}</span>
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
