<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { DropdownMenuContent, DropdownMenuPortal } from 'reka-ui'

import { cn } from '@/lib/utils'

defineOptions({ inheritAttrs: false })

const props = withDefaults(
  defineProps<{
    class?: HTMLAttributes['class']
    sideOffset?: number
    align?: 'start' | 'center' | 'end'
  }>(),
  { sideOffset: 4, align: 'end' },
)
</script>

<template>
  <DropdownMenuPortal>
    <DropdownMenuContent
      v-bind="$attrs"
      :side-offset="props.sideOffset"
      :align="props.align"
      :collision-padding="4"
      :class="
        cn(
          'z-50 min-w-36 overflow-hidden rounded-md border bg-card p-1 text-foreground shadow-md',
          props.class,
        )
      "
    >
      <slot />
    </DropdownMenuContent>
  </DropdownMenuPortal>
</template>
