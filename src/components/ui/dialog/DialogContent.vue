<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { DialogClose, DialogContent, DialogOverlay, DialogPortal } from 'reka-ui'
import { X } from 'lucide-vue-next'

import { t } from '@/i18n'
import { cn } from '@/lib/utils'

defineOptions({ inheritAttrs: false })

const props = defineProps<{ class?: HTMLAttributes['class'] }>()
</script>

<template>
  <DialogPortal>
    <DialogOverlay class="fixed inset-0 z-50 bg-black/25" />
    <DialogContent
      v-bind="$attrs"
      :class="
        cn(
          'fixed top-1/2 left-1/2 z-50 w-[calc(100%-2rem)] max-w-sm -translate-x-1/2 -translate-y-1/2 rounded-lg border bg-card p-4 shadow-lg outline-none',
          props.class,
        )
      "
    >
      <slot />
      <DialogClose
        class="absolute top-3 right-3 inline-flex size-6 items-center justify-center rounded-sm text-muted-foreground hover:bg-accent"
        :title="t('common.close')"
      >
        <X class="size-4" />
      </DialogClose>
    </DialogContent>
  </DialogPortal>
</template>
