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

// [WARN] 这里构建时会报错不能在顶级作用域使用await
// TODO: 最好改一下
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