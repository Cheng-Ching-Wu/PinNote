import { createRouter, createWebHashHistory } from 'vue-router'

import MainView from '@/views/MainView.vue'

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', name: 'main', component: MainView },
    {
      path: '/note/:id',
      name: 'note',
      component: () => import('@/views/NoteView.vue'),
      props: true,
    },
  ],
})
