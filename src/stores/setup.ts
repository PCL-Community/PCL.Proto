import { defineStore } from "pinia";

export interface ISetupOption {
    key: string;
    text: string;
}

export type SetupItemType = 'select' | 'input'

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
        }
    }),
})