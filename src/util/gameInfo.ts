import picCommand from '@/assets/icons/Impulse_Command_Block.gif?url'
import picGrass from '@/assets/icons/Grass_Block_JE7_BE6.png?url'
import picCubblestone from '@/assets/icons/Cobblestone_JE5_BE3.png?url'
import picGold from '@/assets/icons/Block_of_Gold_JE6_BE3.png?url'
import picFabric from '@/assets/icons/Fabric.png?url'
import picForge from '@/assets/icons/Forge.svg?url'
import type { gameVersionType } from '@/api/gameVersions'

export type showIconType = 'command' | 'grass' | 'stone' | 'gold' | pluginType
type showGameType = gameVersionType | 'fool'
export type pluginType =
  | 'vanilla'
  | 'forge'
  | 'neoforge'
  | 'fabric'
  | 'fabric-api'
  | 'quilt'
  | 'laby-mod'
  | 'optifine'

export const gameInfoIcon: Record<showGameType, showIconType> = {
  snapshot: 'command',
  release: 'grass',
  old_beta: 'stone',
  old_alpha: 'stone',
  fool: 'gold',
  old: 'stone',
}

export const showIconPath: Record<showIconType, string> = {
  command: picCommand,
  grass: picGrass,
  stone: picCubblestone,
  gold: picGold,
  neoforge: '',
  fabric: picFabric,
  'fabric-api': picFabric,
  quilt: '',
  'laby-mod': '',
  optifine: '',
  vanilla: picGrass,
  forge: picForge,
}

export const pluginShowText: Record<pluginType, string> = {
  vanilla: 'Minecraft',
  forge: 'Forge',
  neoforge: 'NeoForge',
  fabric: 'Fabric',
  'fabric-api': 'Fabric API',
  quilt: 'Quilt',
  'laby-mod': 'LabyMod',
  optifine: 'OptiFine',
}

export const isIconPixelated: Record<pluginType, boolean> = {
  vanilla: false,
  forge: false,
  neoforge: false,
  fabric: true,
  'fabric-api': true,
  quilt: true,
  'laby-mod': true,
  optifine: true,
}
