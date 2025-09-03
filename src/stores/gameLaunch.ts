import type GameInstance from "@/types/gameInstance";
import type { pluginType } from "../util/gameInfo";
import { defineStore } from "pinia";

export const useSelectedInstance = defineStore('selected-instance', {
    state: () => ({
        name: "Fabulouly Optimized 1.21.4",
        version: "1.21.4",
        // plugins: new Set<pluginType>(["fabric", "fabric-api"]),
        pluginsVersion: {
            'fabric': '0.15.3',
            'fabric-api': '0.15.3',
        } as Record<pluginType, string>,
    } as GameInstance),

    getters: {
        plugins: (state) => Object.keys(state.pluginsVersion) as pluginType[],
    }
})

export const useActiveInstance = defineStore('active-instance', {
    state: () => ([] as GameInstance[])
})