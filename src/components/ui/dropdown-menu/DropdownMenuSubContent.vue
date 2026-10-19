<script setup lang="ts">
import { type HTMLAttributes, useTemplateRef } from 'vue'
import { DropdownMenuPortal, DropdownMenuSubContent } from 'reka-ui'

import { cn } from '@/lib/utils'
import { isPointerOver } from './pointer'

defineOptions({ inheritAttrs: false })

const props = defineProps<{ class?: HTMLAttributes['class'] }>()

const contentRef = useTemplateRef<{ $el?: Element }>('content')

/**
 * In a narrow window the submenu can't fit beside its parent and is pushed over it, often right under
 * the pointer. The trigger then gets a pointerleave, reka-ui moves focus back to the parent menu, and the
 * submenu closes itself as "focus left" — so it never stays open. Keep it open while the pointer is on it.
 */
function onFocusOutside(event: Event) {
  if (isPointerOver(contentRef.value?.$el)) event.preventDefault()
}
</script>

<template>
  <DropdownMenuPortal>
    <DropdownMenuSubContent
      ref="content"
      v-bind="$attrs"
      :side-offset="2"
      :collision-padding="4"
      :class="cn('z-50 max-h-(--reka-dropdown-menu-content-available-height) min-w-36 overflow-x-hidden overflow-y-auto rounded-md border bg-card p-1 text-foreground shadow-md', props.class)"
      @focus-outside="onFocusOutside"
    >
      <slot />
    </DropdownMenuSubContent>
  </DropdownMenuPortal>
</template>
