<script setup lang="ts">
import type { Editor } from '@tiptap/vue-3'
import {
  BetweenHorizontalEnd,
  BetweenVerticalEnd,
  Columns3,
  Rows3,
  Sheet,
  Table,
  Trash2,
} from 'lucide-vue-next'

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { t } from '@/i18n'
import { cn } from '@/lib/utils'

const props = defineProps<{ editor: Editor; buttonClass: string; activeClass: string }>()

const chain = () => props.editor.chain().focus()

function insertTable() {
  chain().insertTable({ rows: 3, cols: 2, withHeaderRow: true }).run()
}
</script>

<template>
  <!-- Outside a table the button inserts one; inside it opens row / column actions. -->
  <DropdownMenu v-if="editor.isActive('table')" :modal="false">
    <DropdownMenuTrigger :class="cn(buttonClass, activeClass)" :title="t('table.title')">
      <Table class="size-3.5" />
    </DropdownMenuTrigger>
    <DropdownMenuContent class="w-40" side="top" align="start" @close-auto-focus.prevent>
      <DropdownMenuItem @select="chain().addRowAfter().run()">
        <BetweenHorizontalEnd />
        {{ t('table.addRow') }}
      </DropdownMenuItem>
      <DropdownMenuItem @select="chain().addColumnAfter().run()">
        <BetweenVerticalEnd />
        {{ t('table.addColumn') }}
      </DropdownMenuItem>
      <DropdownMenuSeparator />
      <DropdownMenuItem @select="chain().deleteRow().run()">
        <Rows3 />
        {{ t('table.deleteRow') }}
      </DropdownMenuItem>
      <DropdownMenuItem @select="chain().deleteColumn().run()">
        <Columns3 />
        {{ t('table.deleteColumn') }}
      </DropdownMenuItem>
      <DropdownMenuSeparator />
      <DropdownMenuItem variant="destructive" @select="chain().deleteTable().run()">
        <Trash2 />
        {{ t('table.delete') }}
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>

  <button v-else type="button" :class="buttonClass" :title="t('table.insert')" @click="insertTable">
    <Sheet class="size-3.5" />
  </button>
</template>
