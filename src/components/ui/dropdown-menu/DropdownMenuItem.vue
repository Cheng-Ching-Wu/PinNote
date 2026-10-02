<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { DropdownMenuItem } from 'reka-ui'

import { cn } from '@/lib/utils'

const props = defineProps<{
  class?: HTMLAttributes['class']
  variant?: 'default' | 'destructive'
}>()
</script>

<template>
  <DropdownMenuItem
    :class="
      cn(
        'flex h-7 items-center gap-2 rounded-sm px-2 text-[13px] outline-none select-none data-highlighted:bg-accent data-disabled:opacity-50 [&_svg]:size-3.5 [&_svg]:text-muted-foreground',
        props.variant === 'destructive' && 'text-destructive data-highlighted:bg-destructive/10 [&_svg]:text-destructive',
        props.class,
      )
    "
  >
    <slot />
  </DropdownMenuItem>
</template>
