<script setup lang="ts">
import { ref } from 'vue'
import { Check, ChevronDown, Trash2 } from 'lucide-vue-next'

import { Button } from '@/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { t } from '@/i18n'
import { cn } from '@/lib/utils'

defineProps<{ count: number }>()

/** Days before trashed notes are deleted for good; 0 = never. */
const retentionDays = defineModel<number>('retentionDays', { required: true })

const emit = defineEmits<{ empty: [] }>()

const OPTIONS = [7, 30, 90, 0] as const
const label = (days: number) => (days === 0 ? t('trash.never') : t('trash.afterDays', { count: days }))

/** Emptying needs a second click. */
const confirming = ref(false)
let resetTimer: ReturnType<typeof setTimeout> | undefined

function empty() {
  if (!confirming.value) {
    confirming.value = true
    clearTimeout(resetTimer)
    resetTimer = setTimeout(() => (confirming.value = false), 4000)
    return
  }
  clearTimeout(resetTimer)
  confirming.value = false
  emit('empty')
}
</script>

<template>
  <div class="flex items-center gap-1 text-[11px] text-muted-foreground">
    <span class="shrink-0">{{ t('trash.autoDelete') }}</span>
    <DropdownMenu :modal="false">
      <DropdownMenuTrigger
        class="inline-flex h-6 items-center gap-0.5 rounded-sm px-1 text-foreground outline-none hover:bg-accent data-[state=open]:bg-accent"
        :title="t('trash.autoDeleteTitle')"
      >
        {{ label(retentionDays) }}
        <ChevronDown class="size-3" />
      </DropdownMenuTrigger>
      <DropdownMenuContent class="w-28" align="start">
        <DropdownMenuItem v-for="days in OPTIONS" :key="days" @select="retentionDays = days">
          <Check :class="cn(days !== retentionDays && 'invisible')" />
          {{ label(days) }}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>

    <Button
      v-if="count > 0"
      variant="ghost-destructive"
      size="sm"
      :class="cn('ml-auto h-6 px-1.5 text-[11px]', confirming && 'bg-destructive/10 font-medium')"
      @click="empty"
    >
      <Trash2 class="size-3" />
      {{ confirming ? t('trash.confirmEmpty', { count }) : t('trash.empty') }}
    </Button>
  </div>
</template>
