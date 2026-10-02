<script setup lang="ts">
import { onBeforeUnmount, reactive, ref, watch } from 'vue'
import { Ban, RotateCcw } from 'lucide-vue-next'

import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'
import { cn } from '@/lib/utils'
import { useShortcutStore } from '@/stores/shortcuts'
import { formatAccelerator, recordAccelerator } from '@/utils/accelerator'

const open = defineModel<boolean>('open', { required: true })

const store = useShortcutStore()

/** Action currently being recorded. */
const recording = ref<string | null>(null)
const preview = ref('')
const errors = reactive<Record<string, string>>({})

async function startRecording(action: string) {
  if (recording.value) await stopRecording()
  errors[action] = ''
  preview.value = ''
  recording.value = action
  // Release all global shortcuts so pressing an existing one reaches this window.
  await store.pause()
  window.addEventListener('keydown', onKeydown, true)
}

async function stopRecording() {
  window.removeEventListener('keydown', onKeydown, true)
  recording.value = null
  preview.value = ''
  await store.resume()
}

async function onKeydown(event: KeyboardEvent) {
  const action = recording.value
  if (!action) return
  // Capture phase on window: nothing else (dialog Esc handling, editor…) sees the key.
  event.preventDefault()
  event.stopPropagation()
  if (event.repeat) return

  const noModifiers = !event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey
  if (event.code === 'Escape' && noModifiers) {
    await stopRecording()
    return
  }

  const result = recordAccelerator(event)
  if (result.kind === 'pending') {
    preview.value = result.preview
    return
  }
  if (result.kind === 'invalid') {
    errors[action] = result.reason
    return
  }

  window.removeEventListener('keydown', onKeydown, true)
  errors[action] = (await store.set(action, result.accelerator)) ?? ''
  await stopRecording()
}

async function apply(action: string, accelerator: string) {
  if (recording.value) await stopRecording()
  errors[action] = (await store.set(action, accelerator)) ?? ''
}

watch(open, (isOpen) => {
  if (isOpen) void store.load()
  else if (recording.value) void stopRecording()
})

onBeforeUnmount(() => {
  if (recording.value) void stopRecording()
})
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent>
      <DialogTitle class="text-[15px] font-semibold">自訂快捷鍵</DialogTitle>
      <DialogDescription class="mt-1 text-xs leading-relaxed text-muted-foreground">
        點擊按鍵欄位後按下想用的按鍵。可用組合鍵（如 Ctrl+Alt+N），或單獨使用 F13–F24 等巨集鍵；按 Esc 取消錄製。
      </DialogDescription>

      <div class="mt-3">
        <div v-for="status in store.statuses" :key="status.action" class="border-b py-2.5 last:border-0">
          <div class="flex items-center gap-1.5">
            <span class="flex-1 text-[13px]">{{ status.label }}</span>
            <button
              type="button"
              :class="
                cn(
                  'h-7 min-w-32 rounded-md border bg-muted/40 px-2 text-xs tabular-nums outline-none hover:bg-accent',
                  !status.accelerator && 'text-muted-foreground',
                  recording === status.action && 'border-ring bg-card ring-2 ring-ring/25',
                )
              "
              :title="recording === status.action ? '按 Esc 取消' : '點擊後按下新的按鍵'"
              @click="recording === status.action ? stopRecording() : startRecording(status.action)"
            >
              {{ recording === status.action ? preview || '請按下按鍵…' : formatAccelerator(status.accelerator) }}
            </button>
            <Button
              variant="ghost"
              size="icon-xs"
              :title="`恢復預設（${formatAccelerator(status.defaultAccelerator)}）`"
              :disabled="status.accelerator === status.defaultAccelerator"
              @click="apply(status.action, status.defaultAccelerator)"
            >
              <RotateCcw class="size-3.5" />
            </Button>
            <Button
              variant="ghost"
              size="icon-xs"
              title="停用此快捷鍵"
              :disabled="!status.accelerator"
              @click="apply(status.action, '')"
            >
              <Ban class="size-3.5" />
            </Button>
          </div>
          <p v-if="errors[status.action]" class="mt-1 text-xs text-destructive">{{ errors[status.action] }}</p>
          <p v-else-if="status.accelerator && !status.registered" class="mt-1 text-xs text-destructive">
            目前無法使用，可能已被其他程式占用，請換一組按鍵
          </p>
        </div>
      </div>
    </DialogContent>
  </Dialog>
</template>
