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

let element: AccountInner | undefined = undefined;

const account = await invoke<Account>('get_account');
for (const key in account) {
    if (Object.prototype.hasOwnProperty.call(account, key)) {
        element = (account as any)[key] as AccountInner;
        break
    }
}

export const useAccountInfo = defineStore('account-info', {
    state: () => (element as AccountInner),
    getters: {
        skinUrl: (state) => getSkinUrl(state.username, 'username')
    }
})