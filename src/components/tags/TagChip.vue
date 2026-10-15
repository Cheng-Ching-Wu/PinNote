<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { X } from 'lucide-vue-next'

import { t } from '@/i18n'
import { cn } from '@/lib/utils'

const props = defineProps<{
  name: string
  /** Dot color (CSS value). */
  color: string
  /** Shows an × button that emits `remove`. */
  removable?: boolean
  class?: HTMLAttributes['class']
}>()

const emit = defineEmits<{ remove: [] }>()
</script>

<template>
  <span
    :class="
      cn(
        'group/tag inline-flex h-5 max-w-40 shrink-0 items-center gap-1 rounded-full border border-current/15 pr-2 pl-1.5 text-[11px] leading-none',
        removable && 'pr-0.5',
        props.class,
      )
    "
    :title="`#${name}`"
  >
    <span class="size-1.5 shrink-0 rounded-full" :style="{ backgroundColor: color }" />
    <span class="truncate">{{ name }}</span>
    <slot />
    <button
      v-if="removable"
      type="button"
      :title="t('tags.remove', { name })"
      class="inline-flex size-4 shrink-0 items-center justify-center rounded-full opacity-50 outline-none hover:bg-current/10 hover:opacity-100 focus-visible:opacity-100"
      @click.stop="emit('remove')"
    >
      <X class="size-2.5" />
    </button>
  </span>
</template>
