//! 实例图标的唯一来源：CE 内置名、图片与选项文案在这里按同一条记录对齐。

import { blockImage } from '@/api/pclAssets'

/// CE 内置方块图标，`name` 同时是 `images/Blocks` 的文件名，`text` 是选项文案键。
export const builtinLogos = [
  { name: 'CobbleStone', text: 'Instance.Overall.Icon.Cobblestone' },
  { name: 'CommandBlock', text: 'Instance.Overall.Icon.CommandBlock' },
  { name: 'GoldBlock', text: 'Instance.Overall.Icon.GoldBlock' },
  { name: 'Grass', text: 'Instance.Overall.Icon.GrassBlock' },
  { name: 'GrassPath', text: 'Instance.Overall.Icon.DirtPath' },
  { name: 'Anvil', text: 'Instance.Overall.Icon.Anvil' },
  { name: 'RedstoneBlock', text: 'Instance.Overall.Icon.RedstoneBlock' },
  { name: 'RedstoneLampOn', text: 'Instance.Overall.Icon.RedstoneLampOn' },
  { name: 'RedstoneLampOff', text: 'Instance.Overall.Icon.RedstoneLampOff' },
  { name: 'Egg', text: 'Instance.Overall.Icon.Egg' },
  { name: 'Fabric', text: 'Instance.Overall.Icon.Fabric' },
  { name: 'Quilt', text: 'Instance.Overall.Icon.Quilt' },
  { name: 'NeoForge', text: 'Instance.Overall.Icon.NeoForgeFox' },
  { name: 'Cleanroom', text: 'Instance.Overall.Icon.Cleanroom' },
]

/// 0 跟随版本类型，1 使用实例内的自定义图片，其余按下标选择内置图标。
export const autoLogo = '0'
export const customLogo = '1'
const firstBuiltinKey = 2
/// 图标只有 CE 目录一份；非打包环境（如直接用 Bun 跑的检查脚本）没有 BASE_URL，退回站点根。
const assetsBase = import.meta.env.BASE_URL ?? '/'

/// 选项编号转换成 CE 配置里保存的 Logo 值。
export function logoFromSelection(selection: string): string {
  if (selection === customLogo) return 'PCL\\Logo.png'
  const logo = builtinLogos[Number(selection) - firstBuiltinKey]
  return logo ? `pack://application:,,,/images/Blocks/${logo.name}.png` : ''
}

/// CE 保存的 Logo 值还原成选项编号，无法识别的自定义图片按自定义处理。
export function selectionFromLogo(stored?: { logo: string; logo_custom: boolean }) {
  if (!stored?.logo_custom || !stored.logo) return autoLogo
  const index = builtinLogos.findIndex((logo) =>
    stored.logo.toLowerCase().endsWith(`/blocks/${logo.name.toLowerCase()}.png`),
  )
  return index < 0 ? customLogo : String(index + firstBuiltinKey)
}

/// 选项编号对应的图标；自动与自定义没有内置图片。
export function imageFromSelection(selection: string) {
  const logo = builtinLogos[Number(selection) - firstBuiltinKey]
  return logo ? blockImage(logo.name, assetsBase) : null
}
