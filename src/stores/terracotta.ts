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

type PlayerProfile = {
  kind: 'LOCAL' | 'HOST' | 'GUEST'
  machine_id: string
  name: string
  vendor: string
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
    profiles: undefined as PlayerProfile[] | undefined,
    difficulty: undefined as string | undefined,
    type: undefined as ExceptionType | undefined,
    avaliable_mc_ports: [] as number[],
    username: 'PCL.Proto Anonymous',
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
    async setHostScanning() {
      console.info('[terracotta] set host scanning')
      try {
        await invoke('set_terracotta_host_scanning')
        this.update()
        return true
      } catch (err) {
        console.error('[terracotta] set host scanning failed', err)
        return false
      }
    },
    async setHostStarting(mcPort: number, player: string) {
      console.info('[terracotta] set host starting', mcPort, player)
      try {
        const roomCode = await invoke<string>('set_terracotta_host_starting', { mcPort, player })
        this.update()
        this.username = player
        return roomCode
      } catch (err) {
        throw err
      }
    },
    async setGuesting(roomCode: string, player: string) {
      console.info('[terracotta] set guesting', roomCode, player)
      try {
        await invoke('set_terracotta_guesting', { roomCode, player })
        this.update()
        this.username = player
        return true
      } catch (err) {
        console.error('[terracotta] set guesting failed', err)
        return false
      }
    },
  },
})

export default useTerracottaStore
