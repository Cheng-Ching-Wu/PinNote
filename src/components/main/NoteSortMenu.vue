<script setup lang="ts">
import { ArrowDownUp, Check } from 'lucide-vue-next'

import { buttonVariants } from '@/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { cn } from '@/lib/utils'
import { NOTE_SORT_LABELS, NOTE_SORTS } from '@/utils/sort'
import type { NoteSort } from '@/utils/sort'

const sort = defineModel<NoteSort>({ required: true })
</script>

<template>
  <DropdownMenu :modal="false">
    <DropdownMenuTrigger
      :class="cn(buttonVariants({ variant: 'ghost', size: 'sm' }), 'h-6 gap-1 px-1.5 text-[11px] font-normal text-muted-foreground')"
      title="排序方式"
    >
      <ArrowDownUp class="size-3" />
      {{ NOTE_SORT_LABELS[sort] }}
    </DropdownMenuTrigger>
    <DropdownMenuContent class="w-36">
      <DropdownMenuItem v-for="option in NOTE_SORTS" :key="option" @select="sort = option">
        <Check :class="cn(option !== sort && 'invisible')" />
        {{ NOTE_SORT_LABELS[option] }}
        <span v-if="option === 'manual'" class="ml-auto text-[10px] text-muted-foreground">拖曳</span>
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
