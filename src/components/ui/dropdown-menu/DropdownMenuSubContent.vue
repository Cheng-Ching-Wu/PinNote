<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { DropdownMenuPortal, DropdownMenuSubContent } from 'reka-ui'

import { cn } from '@/lib/utils'

defineOptions({ inheritAttrs: false })

const props = defineProps<{ class?: HTMLAttributes['class'] }>()
</script>

<template>
  <DropdownMenuPortal>
    <DropdownMenuSubContent
      v-bind="$attrs"
      :side-offset="2"
      :collision-padding="4"
      :class="cn('z-50 min-w-36 overflow-hidden rounded-md border bg-card p-1 text-foreground shadow-md', props.class)"
    >
      <slot />
    </DropdownMenuSubContent>
  </DropdownMenuPortal>
</template>
