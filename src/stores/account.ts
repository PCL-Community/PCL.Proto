// import getSkinUrl from "@/util/skinGetter";
import { invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'
import { ref } from 'vue'
import SteveSkin from '/default-skin/Steve_(classic_texture)_JE6.png'

interface AccountInner {
  username: string
  uuid: string
}
interface Account {
  Offline?: AccountInner
}

export const useAccountInfo = defineStore('account-info', () => {
  const username = ref<string>()
  const uuid = ref<string>()

  async function initialize() {
    try {
      const account = await invoke<Account>('get_account')
      for (const key in account) {
        if (Object.prototype.hasOwnProperty.call(account, key)) {
          const element = (account as any)[key] as AccountInner
          username.value = element.username
          uuid.value = element.uuid
          break
        }
      }
    } catch (error) {
      console.error('Failed to initialize account:', error)
    }
  }

  return {
    username,
    uuid,
    initialize,
  }
})
