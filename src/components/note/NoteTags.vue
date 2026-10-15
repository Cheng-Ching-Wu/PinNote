<script setup lang="ts">
import { nextTick, useTemplateRef, watch } from 'vue'
import { Plus } from 'lucide-vue-next'

import TagChip from '@/components/tags/TagChip.vue'
import TagInput from '@/components/tags/TagInput.vue'
import { useTagColors } from '@/composables/useTagColors'
import { t } from '@/i18n'
import { cn } from '@/lib/utils'

const props = defineProps<{
  tags: readonly string[]
  suggestions: readonly string[]
}>()

/** Whether the tag input is shown (also opened from the note menu). */
const editing = defineModel<boolean>('editing', { required: true })

const emit = defineEmits<{ change: [tags: string[]] }>()

const { tagColorValue } = useTagColors()
const inputRef = useTemplateRef<InstanceType<typeof TagInput>>('inputRef')

watch(editing, async (value) => {
  if (!value) return
  await nextTick()
  // Next frame: when opened from the note menu, let the menu finish closing first.
  requestAnimationFrame(() => inputRef.value?.focus())
})

function add(name: string) {
  emit('change', [...props.tags, name])
}

function remove(index: number) {
  emit('change', props.tags.filter((_, i) => i !== index))
}
</script>

<template>
  <!--
    Faded until the pointer is over the note so tags don't compete with the text;
    fully visible while editing.
  -->
  <div
    v-if="tags.length > 0 || editing"
    :class="
      cn(
        'flex shrink-0 flex-wrap items-center gap-1 px-2.5 pt-0.5 pb-1.5 text-note-ink transition-opacity duration-200',
        editing ? 'opacity-100' : 'opacity-45 group-hover/note:opacity-100 focus-within:opacity-100',
      )
    "
  >
    <TagChip
      v-for="(tag, index) in tags"
      :key="tag"
      :name="tag"
      :color="tagColorValue(tag)"
      removable
      class="bg-note-header/60"
      @remove="remove(index)"
    />
    <TagInput
      v-if="editing"
      ref="inputRef"
      :current="tags"
      :suggestions="suggestions"
      :tag-color="tagColorValue"
      placement="top"
      class="w-24 placeholder:text-note-ink/40"
      @add="add"
      @remove-last="tags.length > 0 && remove(tags.length - 1)"
      @done="editing = false"
    />
    <button
      v-else
      type="button"
      :title="t('common.addTag')"
      class="inline-flex size-5 items-center justify-center rounded-full text-note-ink/60 outline-none hover:bg-note-ink/10 hover:text-note-ink focus-visible:bg-note-ink/10"
      @click="editing = true"
    >
      <Plus class="size-3" />
    </button>
  </div>
</template>
