import { blockImage } from '@/api/pclAssets'
import type { gameVersionType } from '@/api/gameVersions'

export type showIconType = 'command' | 'grass' | 'stone' | 'gold' | pluginType
type showGameType = gameVersionType | 'fool'
export type pluginType =
  | 'vanilla'
  | 'forge'
  | 'neoforge'
  | 'fabric'
  | 'legacyfabric'
  | 'liteloader'
  | 'cleanroom'
  | 'fabric-api'
  | 'quilt'
  | 'laby-mod'
  | 'optifine'

export const gameInfoIcon: Record<showGameType, showIconType> = {
  snapshot: 'command',
  special: 'gold',
  pending: 'command',
  release: 'grass',
  old_beta: 'stone',
  old_alpha: 'stone',
  fool: 'gold',
  old: 'stone',
}

/// 图标只有 CE 目录一份；非打包环境（如直接用 Bun 跑的检查脚本）没有 BASE_URL，退回站点根。
const assetsBase = import.meta.env.BASE_URL ?? '/'

export const showIconPath: Record<showIconType, string> = {
  command: blockImage('CommandBlock', assetsBase),
  grass: blockImage('Grass', assetsBase),
  stone: blockImage('CobbleStone', assetsBase),
  gold: blockImage('GoldBlock', assetsBase),
  neoforge: blockImage('NeoForge', assetsBase),
  fabric: blockImage('Fabric', assetsBase),
  legacyfabric: blockImage('Fabric', assetsBase),
  liteloader: blockImage('Egg', assetsBase),
  cleanroom: blockImage('Cleanroom', assetsBase),
  'fabric-api': blockImage('Fabric', assetsBase),
  quilt: blockImage('Quilt', assetsBase),
  'laby-mod': blockImage('LabyMod', assetsBase),
  optifine: blockImage('GrassPath', assetsBase),
  vanilla: blockImage('Grass', assetsBase),
  forge: blockImage('Anvil', assetsBase),
}

export const instanceLoaderOrder: pluginType[] = [
  'forge',
  'neoforge',
  'cleanroom',
  'laby-mod',
  'quilt',
  'fabric',
  'legacyfabric',
  'liteloader',
]

export function instanceLoader(plugins: Partial<Record<pluginType, string>>) {
  return instanceLoaderOrder.find((key) => plugins[key])
}

export function instanceIcon(
  version: string,
  plugins: Partial<Record<pluginType, string>>,
  versionType?: string,
) {
  const key = instanceLoader(plugins) || (plugins.optifine ? 'optifine' : undefined)
  if (key) return showIconPath[key]
  if (versionType && Object.prototype.hasOwnProperty.call(gameInfoIcon, versionType))
    return showIconPath[gameInfoIcon[versionType as showGameType]]
  if (/^\d{2}w\d{2}[a-z]/i.test(version)) return showIconPath.command
  if (/^(?:a\d|b\d|c\d|rd-)/i.test(version)) return showIconPath.stone
  return showIconPath.grass
}

export function loaderIcon(name: string) {
  const keys: Record<string, showIconType> = {
    forge: 'forge',
    neoforge: 'neoforge',
    cleanroom: 'cleanroom',
    fabric: 'fabric',
    legacyfabric: 'legacyfabric',
    fabricapi: 'fabric-api',
    legacyfabricapi: 'fabric-api',
    quilt: 'quilt',
    labymod: 'laby-mod',
    liteloader: 'liteloader',
    optifine: 'optifine',
    optifabric: 'optifine',
  }
  return showIconPath[keys[name.toLowerCase().replace(/[^a-z]/g, '')] || 'grass']
}

export const pluginShowText: Record<pluginType, string> = {
  vanilla: 'Minecraft',
  forge: 'Forge',
  neoforge: 'NeoForge',
  fabric: 'Fabric',
  legacyfabric: 'Legacy Fabric',
  liteloader: 'LiteLoader',
  cleanroom: 'Cleanroom',
  'fabric-api': 'Fabric API',
  quilt: 'Quilt',
  'laby-mod': 'LabyMod',
  optifine: 'OptiFine',
}
