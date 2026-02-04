import { defineStore } from 'pinia'
import { ref } from 'vue'

export enum TerracottaState {
  Idle = "Idle",
  HostScanning = "HostScanning",
  HostStarting = "HostStarting",
  HostOk = "HostOk",
  GuestConnecting = "GuestConnecting",
  GuestStarting = "GuestStarting",
  GuestOk = "GuestOk",
  Exception = "Exception",
}

export enum PlayerType {
  Host = "host",
  Guest = "guest",
}

export const useScaffolding = defineStore('scaffolding', () => {
  const roomCode = ref<string | undefined>(undefined)
  const terracotta_state = ref<TerracottaState>(TerracottaState.Idle)
  const playerType = ref<PlayerType | undefined>(undefined)
  return { roomCode, terracotta_state, playerType }
})
