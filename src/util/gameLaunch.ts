import { reactive } from "vue";
import type { pluginType } from "./gameInfo";

export const selectedInstance = reactive({
    name: "Fabulouly Optimized 1.21.4",
    version: "1.21.4",
    plugins: ["fabric", "fabric-api"] as pluginType[],
    pluginsVersion: {
        'fabric': '0.15.3',
        'fabric-api': '0.15.3',
    } as Record<pluginType, string>,
})