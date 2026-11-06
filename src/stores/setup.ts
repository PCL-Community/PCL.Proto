import { defineStore } from 'pinia'

export const useSetup = defineStore('setup', {
  state: () => ({
    launch: {
      defaultVersionIsolation: 'isolate-all',
      gameWindowTitle: '',
      customInfo: 'PCL.Proto',
      launcherVisibility: 'close-immediately',
    },
    gameMemory: {
      mode: 'auto' as 'auto' | 'custom',
      percent: 0,
      optimizeBeforeStart: true,
    },
  }),
})
