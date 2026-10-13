<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { DropdownMenuSubTrigger } from 'reka-ui'
import { ChevronRight } from 'lucide-vue-next'

import { cn } from '@/lib/utils'

const props = defineProps<{ class?: HTMLAttributes['class'] }>()
</script>

<template>
  <DropdownMenuSubTrigger
    :class="
      cn(
        'flex h-7 items-center gap-2 rounded-sm px-2 text-[13px] outline-none select-none data-highlighted:bg-accent data-[state=open]:bg-accent [&_svg]:size-3.5 [&_svg]:text-muted-foreground',
        props.class,
      )
    "
  >
    <slot />
    <ChevronRight class="ml-auto" />
  </DropdownMenuSubTrigger>
</template>
