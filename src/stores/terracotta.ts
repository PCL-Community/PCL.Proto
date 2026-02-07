import { invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'

enum ExceptionType {
  PingHostFail = 0,
  PingHostRst = 1,
  GuestEasytierCrash = 2,
  HostEasytierCrash = 3,
  PingServerRst = 4,
  ScaffoldingInvalidResponse = 5,
}

const useTerracottaStore = defineStore('terracotta', {
  state: () => ({
    state: 'waiting' as
      | 'waiting'
      | 'host-scanning'
      | 'host-starting'
      | 'guest-connecting'
      | 'host-ok'
      | 'guest-ok'
      | 'guest-starting'
      | 'exception',
    index: 0,
    room: undefined as string | undefined,
    url: undefined as string | undefined,
    profile_index: undefined as number | undefined,
    profiles: undefined as [] | undefined,
    difficulty: undefined as string | undefined,
    type: undefined as ExceptionType | undefined,
    autoUpdateEnabled: false,
    autoUpdateInterval: 2000,
    autoUpdateTimerId: null as number | null, // 自动更新定时器 ID 用于停止自动更新
  }),
  actions: {
    update() {
      invoke('get_terracotta_state').then((state: any) => {
        this.$patch(state)
        console.info('[terracotta] state updated', state)
      })
    },
    startAutoUpdate() {
      if (this.autoUpdateTimerId) {
        clearInterval(this.autoUpdateTimerId)
      }
      this.autoUpdateEnabled = true
      this.autoUpdateTimerId = window.setInterval(() => {
        this.update()
      }, this.autoUpdateInterval)
      console.info('[terracotta] auto update started', { interval: this.autoUpdateInterval })
    },
    stopAutoUpdate() {
      if (this.autoUpdateTimerId) {
        clearInterval(this.autoUpdateTimerId)
        this.autoUpdateTimerId = null
      }
      this.autoUpdateEnabled = false
      console.info('[terracotta] auto update stopped')
    },
    setWaiting() {
      console.info('[terracotta] set waiting')
      invoke('set_terracotta_waiting').then(() => {
        this.update()
      })
    },
    setHostScanning(player: string) {
      console.info('[terracotta] set host scanning', player)
      invoke('set_terracotta_host_scanning', { player }).then(() => {
        this.update()
      })
    },
    setGuesting(roomCode: string, player: string) {
      console.info('[terracotta] set guesting', roomCode, player)
      invoke('set_terracotta_guesting', { roomCode, player }).then(() => {
        this.update()
      })
    },
  },
})

export default useTerracottaStore
