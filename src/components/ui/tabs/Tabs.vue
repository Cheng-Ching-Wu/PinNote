<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { TabsRoot } from 'reka-ui'

import { cn } from '@/lib/utils'

const props = defineProps<{ class?: HTMLAttributes['class'] }>()
</script>

<template>
  <TabsRoot :class="cn('flex flex-col', props.class)">
    <slot />
  </TabsRoot>
</template>
