<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { TabsTrigger } from 'reka-ui'

import { cn } from '@/lib/utils'

const props = defineProps<{ value: string; class?: HTMLAttributes['class'] }>()
</script>

<template>
  <TabsTrigger
    :value="props.value"
    :class="
      cn(
        'inline-flex h-full flex-1 items-center justify-center gap-1 rounded-[5px] px-2 text-xs font-medium transition-colors outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring/40 data-[state=active]:bg-card data-[state=active]:text-foreground data-[state=active]:shadow-xs',
        props.class,
      )
    "
  >
    <slot />
  </TabsTrigger>
</template>
