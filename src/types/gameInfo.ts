import { instancePictures } from '@/api/instanceLogo'
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

export const showIconPath: Record<showIconType, string> = {
  command: instancePictures.commandBlock,
  grass: instancePictures.grassBlock,
  stone: instancePictures.cobbleStone,
  gold: instancePictures.goldBlock,
  neoforge: instancePictures.neoForge,
  fabric: instancePictures.fabric,
  legacyfabric: instancePictures.fabric,
  liteloader: instancePictures.egg,
  cleanroom: instancePictures.cleanroom,
  'fabric-api': instancePictures.fabric,
  quilt: instancePictures.quilt,
  'laby-mod': instancePictures.labyMod,
  optifine: instancePictures.grassPath,
  vanilla: instancePictures.grassBlock,
  forge: instancePictures.anvil,
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

export const isIconPixelated: Record<pluginType, boolean> = {
  vanilla: false,
  forge: false,
  neoforge: false,
  fabric: true,
  legacyfabric: true,
  liteloader: true,
  cleanroom: true,
  'fabric-api': true,
  quilt: true,
  'laby-mod': true,
  optifine: true,
}
