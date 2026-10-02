import { createApp } from 'vue'
import { createPinia } from 'pinia'

import App from './App.vue'
import { router } from './router'
import { windowService } from './services/windowService'
import { useNoteStore } from './stores/note'
import './style.css'

async function bootstrap() {
  const app = createApp(App)
  app.use(createPinia())
  app.use(router)

  // Every window loads the same bundle; note windows are identified by their label (`note-{id}`).
  const noteId = windowService.currentNoteId()
  await router.replace(noteId ? { name: 'note', params: { id: noteId } } : { name: 'main' })

  await useNoteStore().startSync()
  await router.isReady()
  app.mount('#app')
}

void bootstrap()
