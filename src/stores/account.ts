import getSkinUrl from "@/util/skinGetter";
import { defineStore } from "pinia";

export const useAccountInfo = defineStore('account-info', {
    state: () => ({
        username: "AMagicPear",
        uuid: 'uuid',
    }),
    getters: {
        skinUrl: (state) => getSkinUrl(state.username, 'username')
    }
})