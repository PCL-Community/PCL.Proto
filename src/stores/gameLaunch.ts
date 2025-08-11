import type { pluginType } from "../util/gameInfo";
import { defineStore } from "pinia";

export const useSelectedInstance = defineStore('selected-instance', {
    state: () => ({
        name: "Fabulouly Optimized 1.21.4",
        version: "1.21.4",
        plugins: ["fabric", "fabric-api"] as pluginType[],
        pluginsVersion: {
            'fabric': '0.15.3',
            'fabric-api': '0.15.3',
        } as Record<pluginType, string>,
    })
})