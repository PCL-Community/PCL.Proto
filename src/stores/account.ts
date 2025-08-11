import { defineStore } from "pinia";

export const useAccountInfo = defineStore('account-info', {
    state: () => ({
        username: "AMagicPear",
        uuid: 'uuid',
    })
})