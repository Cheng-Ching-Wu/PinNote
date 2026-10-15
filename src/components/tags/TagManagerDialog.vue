<script setup lang="ts">
import { Check, Trash2 } from 'lucide-vue-next'

import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { TAG_COLOR_VALUES, TAG_COLORS, type TagColor } from '@/constants/tagColors'
import { useTagColors } from '@/composables/useTagColors'
import { t } from '@/i18n'
import { cn } from '@/lib/utils'
import { useNoteStore } from '@/stores/note'
import { useSettingsStore } from '@/stores/settings'
import { MAX_TAG_LENGTH, normalizeTag, type TagSummary } from '@/utils/tags'

const open = defineModel<boolean>('open', { required: true })

const store = useNoteStore()
const settings = useSettingsStore()
const { tagColor, tagColorValue } = useTagColors()

/** Whether the tag uses a color picked by the user (vs. the automatic one). */
const hasCustomColor = (tag: TagSummary) => tag.key in settings.settings.tagColors

function setColor(tag: TagSummary, color: TagColor | null) {
  void settings.setTagColor(tag.name, color)
}

/** Commits the edited name on Enter / blur; an empty name restores the old one. */
async function rename(tag: TagSummary, event: Event) {
  const input = event.target as HTMLInputElement
  const name = normalizeTag(input.value)
  if (!name || name === tag.name) {
    input.value = tag.name
    return
  }
  // The renamed notes (and the moved color) arrive through the sync events.
  await store.renameTag(tag.name, name)
}

/** Escape puts the old name back (instead of closing the dialog). */
function cancelRename(tag: TagSummary, event: Event) {
  const input = event.target as HTMLInputElement
  input.value = tag.name
  input.blur()
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="max-w-md">
      <DialogTitle class="text-[15px] font-semibold">{{ t('tags.managerTitle') }}</DialogTitle>
      <DialogDescription class="mt-1 text-xs leading-relaxed text-muted-foreground">
        {{ t('tags.managerDescription') }}
      </DialogDescription>

      <p v-if="store.allTags.length === 0" class="py-8 text-center text-xs text-muted-foreground">
        {{ t('tags.none') }}
      </p>
      <ul v-else class="mt-3 max-h-80 overflow-y-auto">
        <li v-for="tag in store.allTags" :key="tag.key" class="flex items-center gap-1.5 border-b py-1.5 last:border-0">
          <DropdownMenu :modal="false">
            <DropdownMenuTrigger
              class="inline-flex size-6 shrink-0 items-center justify-center rounded-sm outline-none hover:bg-accent data-[state=open]:bg-accent"
              :title="t('common.changeColor')"
            >
              <span class="size-2.5 rounded-full" :style="{ backgroundColor: tagColorValue(tag.name) }" />
            </DropdownMenuTrigger>
            <DropdownMenuContent class="w-40">
              <div class="grid grid-cols-5 gap-1.5 p-1.5">
                <button
                  v-for="color in TAG_COLORS"
                  :key="color"
                  type="button"
                  :title="t(`tagColors.${color}`)"
                  :class="
                    cn(
                      'size-5 rounded-full transition-transform hover:scale-110',
                      hasCustomColor(tag) &&
                        tagColor(tag.name) === color &&
                        'ring-2 ring-ring/60 ring-offset-1 ring-offset-card',
                    )
                  "
                  :style="{ backgroundColor: TAG_COLOR_VALUES[color] }"
                  @click="setColor(tag, color)"
                />
              </div>
              <DropdownMenuItem @select="setColor(tag, null)">
                <Check :class="cn(hasCustomColor(tag) && 'invisible')" />
                {{ t('tags.autoColor') }}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>

          <input
            :value="tag.name"
            :maxlength="MAX_TAG_LENGTH"
            spellcheck="false"
            class="h-7 min-w-0 flex-1 rounded-sm border border-transparent bg-transparent px-1.5 text-[13px] outline-none hover:border-input focus:border-ring focus:ring-2 focus:ring-ring/20"
            :title="t('tags.rename')"
            @keydown.enter.prevent="($event.target as HTMLInputElement).blur()"
            @keydown.esc.stop.prevent="cancelRename(tag, $event)"
            @blur="rename(tag, $event)"
          />
          <span class="w-12 shrink-0 text-right text-[11px] text-muted-foreground tabular-nums">
            {{ t('tags.noteCount', { count: tag.count }) }}
          </span>
          <Button
            variant="ghost-destructive"
            size="icon-xs"
            :title="t('tags.delete', { name: tag.name })"
            @click="store.deleteTag(tag.name)"
          >
            <Trash2 class="size-3.5" />
          </Button>
        </li>
      </ul>
    </DialogContent>
  </Dialog>
</template>
