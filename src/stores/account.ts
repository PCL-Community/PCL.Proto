import getSkinUrl from "@/util/skinGetter";
import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";

interface AccountInner {
    username: string,
    uuid: string,
}
interface Account {
    Offline?: AccountInner,
}

export const useAccountInfo = defineStore('account-info', {
    state: () => ({
        username: undefined as string | undefined,
        uuid: undefined as string | undefined,
    }),
    getters: {
        skinUrl: (state) => getSkinUrl(state.username as string, 'username')
    },
    actions: {
        async initialize() {
            try {
                const account = await invoke<Account>('get_account');
                for (const key in account) {
                    if (Object.prototype.hasOwnProperty.call(account, key)) {
                        const element = (account as any)[key] as AccountInner;
                        this.username = element.username;
                        this.uuid = element.uuid;
                        break;
                    }
                }
            } catch (error) {
                console.error('Failed to initialize account:', error);
            }
        }
    }
})