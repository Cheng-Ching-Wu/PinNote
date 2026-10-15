<script setup lang="ts">
import { ref } from 'vue'
import { Archive, ArchiveRestore, Ellipsis, ExternalLink, RotateCcw, Tag, Trash2 } from 'lucide-vue-next'

import { buttonVariants } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { t } from '@/i18n'
import { cn } from '@/lib/utils'
import type { NoteStatus } from '@/models/note'

defineProps<{ status: NoteStatus }>()

const emit = defineEmits<{
  open: []
  'edit-tags': []
  archive: []
  restore: []
  delete: []
  'delete-permanently': []
}>()

const open = ref(false)
/** Permanent deletion needs a second click; reset whenever the menu closes. */
const confirming = ref(false)

function onOpenChange(value: boolean) {
  open.value = value
  if (!value) confirming.value = false
}

function deletePermanently(event: Event) {
  if (!confirming.value) {
    event.preventDefault() // keep the menu open for the confirmation click
    confirming.value = true
    return
  }
  emit('delete-permanently')
}
</script>

<template>
  <DropdownMenu :open="open" :modal="false" @update:open="onOpenChange">
    <DropdownMenuTrigger
      :class="
        cn(
          buttonVariants({ variant: 'ghost', size: 'icon-xs' }),
          'text-muted-foreground opacity-0 group-hover:opacity-100 focus-visible:opacity-100 data-[state=open]:bg-accent data-[state=open]:opacity-100',
        )
      "
      :title="t('item.moreActions')"
      @click.stop
      @keydown.enter.stop
    >
      <Ellipsis class="size-4" />
    </DropdownMenuTrigger>

    <DropdownMenuContent class="w-40" @click.stop>
      <template v-if="status === 'active'">
        <DropdownMenuItem @select="emit('open')">
          <ExternalLink />
          {{ t('item.open') }}
        </DropdownMenuItem>
        <DropdownMenuItem @select="emit('edit-tags')">
          <Tag />
          {{ t('common.editTags') }}
        </DropdownMenuItem>
        <DropdownMenuItem @select="emit('archive')">
          <Archive />
          {{ t('common.archive') }}
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem variant="destructive" @select="emit('delete')">
          <Trash2 />
          {{ t('common.moveToTrash') }}
        </DropdownMenuItem>
      </template>

      <template v-else-if="status === 'archived'">
        <DropdownMenuItem @select="emit('restore')">
          <ArchiveRestore />
          {{ t('common.unarchive') }}
        </DropdownMenuItem>
        <DropdownMenuItem @select="emit('edit-tags')">
          <Tag />
          {{ t('common.editTags') }}
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem variant="destructive" @select="emit('delete')">
          <Trash2 />
          {{ t('common.moveToTrash') }}
        </DropdownMenuItem>
      </template>

      <template v-else>
        <DropdownMenuItem @select="emit('restore')">
          <RotateCcw />
          {{ t('common.restore') }}
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem
          variant="destructive"
          :class="confirming && 'bg-destructive/10 font-medium'"
          @select="deletePermanently"
        >
          <Trash2 />
          {{ confirming ? t('common.confirmDeletePermanently') : t('common.deletePermanently') }}
        </DropdownMenuItem>
      </template>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
