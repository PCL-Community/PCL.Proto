import type { pluginType } from "@/util/gameInfo"

export default interface GameInstance {
    name: string
    version: string
    plugins: pluginType[]
    pluginsVersion: Record<pluginType, string>
}

