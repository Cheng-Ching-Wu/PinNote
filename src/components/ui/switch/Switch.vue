<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { SwitchRoot, SwitchThumb } from 'reka-ui'

import { cn } from '@/lib/utils'

const props = defineProps<{ class?: HTMLAttributes['class'] }>()

const checked = defineModel<boolean>({ required: true })
</script>

<template>
  <SwitchRoot
    v-model="checked"
    :class="
      cn(
        'inline-flex h-4 w-7 shrink-0 items-center rounded-full border border-transparent bg-input transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring/50 data-[state=checked]:bg-primary',
        props.class,
      )
    "
  >
    <SwitchThumb
      class="block size-3 rounded-full bg-card shadow-sm transition-transform data-[state=checked]:translate-x-3 data-[state=unchecked]:translate-x-0.5"
    />
  </SwitchRoot>
</template>
