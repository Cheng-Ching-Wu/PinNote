<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { TabsList } from 'reka-ui'

import { cn } from '@/lib/utils'

const props = defineProps<{ class?: HTMLAttributes['class'] }>()
</script>

<template>
  <TabsList :class="cn('inline-flex h-8 items-center rounded-md bg-muted p-0.5 text-muted-foreground', props.class)">
    <slot />
  </TabsList>
</template>
