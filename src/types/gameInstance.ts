import type { pluginType } from '@/types/gameInfo'

// export default interface GameInstance {
//     name: string
//     version: string
//     // plugins: pluginType[]
//     pluginsVersion: Record<pluginType, string>
// }
export default interface GameInstance {
  id: string
  name: string
  directory: string
  jar_path: string
  version: string
  json_path: string
  natives_path: string
  // game_java: GameJava,
  global_dir: string
  pluginsVersion: Partial<Record<pluginType, string>>
}

export interface GameRepository {
  name: string
  path: string
}
