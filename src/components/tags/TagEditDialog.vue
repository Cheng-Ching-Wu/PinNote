<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from 'vue'

import TagChip from '@/components/tags/TagChip.vue'
import TagInput from '@/components/tags/TagInput.vue'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'
import { useTagColors } from '@/composables/useTagColors'
import { t } from '@/i18n'
import type { Note } from '@/models/note'
import { useNoteStore } from '@/stores/note'
import { tagKey } from '@/utils/tags'

/** Notes whose tags are edited (one, or a multi-selection); the dialog is open while non-empty. */
const noteIds = defineModel<string[]>('noteIds', { required: true })

const store = useNoteStore()
const { tagColorValue } = useTagColors()
const inputRef = useTemplateRef<InstanceType<typeof TagInput>>('inputRef')

const notes = computed(() => noteIds.value.map((id) => store.getById(id)).filter((n): n is Note => !!n))

/** Tags on any of the notes, in first-seen order, with how many of the notes have them. */
const tags = computed(() => {
  const byKey = new Map<string, { name: string; count: number }>()
  for (const note of notes.value) {
    for (const tag of note.tags) {
      const entry = byKey.get(tagKey(tag))
      if (entry) entry.count++
      else byKey.set(tagKey(tag), { name: tag, count: 1 })
    }
  }
  return [...byKey.values()]
})
/** Tags every note already has — left out of the suggestions; partial ones can still be added to all. */
const sharedTags = computed(() => tags.value.filter((tag) => tag.count === notes.value.length).map((tag) => tag.name))
const suggestions = computed(() => store.allTags.map((tag) => tag.name))

const open = computed({
  get: () => notes.value.length > 0,
  set: (value) => {
    if (!value) noteIds.value = []
  },
})

watch(open, async (value) => {
  if (!value) return
  await nextTick()
  inputRef.value?.focus()
})

const add = (name: string) => store.addTagToNotes(noteIds.value, name)
const remove = (name: string) => store.removeTagFromNotes(noteIds.value, name)

function removeLast() {
  const last = tags.value.at(-1)
  if (last) void remove(last.name)
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent @open-auto-focus.prevent>
      <DialogTitle class="text-[15px] font-semibold">
        {{ notes.length > 1 ? t('tags.editTitleMany', { count: notes.length }) : t('tags.editTitle') }}
      </DialogTitle>
      <DialogDescription class="mt-1 text-xs text-muted-foreground">
        {{ t('tags.editDescription') }}
        <template v-if="notes.length > 1">{{ t('tags.editDescriptionMany') }}</template>
      </DialogDescription>

      <div
        class="mt-3 flex min-h-9 flex-wrap items-center gap-1 rounded-md border border-input px-2 py-1.5 focus-within:border-ring focus-within:ring-2 focus-within:ring-ring/20"
        @click="inputRef?.focus()"
      >
        <TagChip
          v-for="tag in tags"
          :key="tag.name"
          :name="tag.name"
          :color="tagColorValue(tag.name)"
          removable
          :class="tag.count < notes.length && 'border-dashed'"
          @remove="remove(tag.name)"
        >
          <span v-if="tag.count < notes.length" class="text-[10px] text-muted-foreground tabular-nums">
            {{ tag.count }}/{{ notes.length }}
          </span>
        </TagChip>
        <TagInput
          ref="inputRef"
          :current="sharedTags"
          :suggestions="suggestions"
          :tag-color="tagColorValue"
          class="text-[12px]"
          @add="add"
          @remove-last="removeLast"
        />
      </div>
    </DialogContent>
  </Dialog>
</template>
