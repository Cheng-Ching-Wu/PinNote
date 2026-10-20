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
  <!-- max-h uses the space reka-ui measures inside the window, so a tall menu scrolls instead of being cut off. -->
  <DropdownMenuPortal>
    <DropdownMenuContent
      v-bind="$attrs"
      :side-offset="props.sideOffset"
      :align="props.align"
      :collision-padding="4"
      :class="
        cn(
          'z-50 max-h-(--reka-dropdown-menu-content-available-height) min-w-36 overflow-x-hidden overflow-y-auto rounded-md border bg-card p-1 text-foreground shadow-md',
          props.class,
        )
      "
    >
      <slot />
    </DropdownMenuContent>
  </DropdownMenuPortal>
</template>
