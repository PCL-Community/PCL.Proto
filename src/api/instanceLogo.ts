//! 实例图标的唯一来源：CE 内置名、渲染图片和下拉文案在这里按同一条记录对齐。

import anvil from '@/assets/icons/instance/Anvil.png?url'
import cleanroom from '@/assets/icons/instance/Cleanroom.png?url'
import cobbleStone from '@/assets/icons/instance/Cobblestone_JE5_BE3.png?url'
import commandBlock from '@/assets/icons/instance/Impulse_Command_Block.gif?url'
import egg from '@/assets/icons/instance/Egg.png?url'
import fabric from '@/assets/icons/instance/Fabric.png?url'
import goldBlock from '@/assets/icons/instance/Block_of_Gold_JE6_BE3.png?url'
import grassBlock from '@/assets/icons/instance/Grass_Block_JE7_BE6.png?url'
import grassPath from '@/assets/icons/instance/GrassPath.png?url'
import labyMod from '@/assets/icons/instance/LabyMod.png?url'
import neoForge from '@/assets/icons/instance/NeoForge.png?url'
import quilt from '@/assets/icons/instance/Quilt.png?url'
import redstoneBlock from '@/assets/icons/instance/RedstoneBlock.png?url'
import redstoneLampOff from '@/assets/icons/instance/RedstoneLampOff.png?url'
import redstoneLampOn from '@/assets/icons/instance/RedstoneLampOn.png?url'

/// 启动器内置实例图片，`name` 为 CE 的 images/Blocks 文件名，`text` 为选项文案键。
export const builtinLogos = [
  { name: 'CobbleStone', text: 'Instance.Overall.Icon.Cobblestone', image: cobbleStone },
  { name: 'CommandBlock', text: 'Instance.Overall.Icon.CommandBlock', image: commandBlock },
  { name: 'GoldBlock', text: 'Instance.Overall.Icon.GoldBlock', image: goldBlock },
  { name: 'Grass', text: 'Instance.Overall.Icon.GrassBlock', image: grassBlock },
  { name: 'GrassPath', text: 'Instance.Overall.Icon.DirtPath', image: grassPath },
  { name: 'Anvil', text: 'Instance.Overall.Icon.Anvil', image: anvil },
  { name: 'RedstoneBlock', text: 'Instance.Overall.Icon.RedstoneBlock', image: redstoneBlock },
  { name: 'RedstoneLampOn', text: 'Instance.Overall.Icon.RedstoneLampOn', image: redstoneLampOn },
  {
    name: 'RedstoneLampOff',
    text: 'Instance.Overall.Icon.RedstoneLampOff',
    image: redstoneLampOff,
  },
  { name: 'Egg', text: 'Instance.Overall.Icon.Egg', image: egg },
  { name: 'Fabric', text: 'Instance.Overall.Icon.Fabric', image: fabric },
  { name: 'Quilt', text: 'Instance.Overall.Icon.Quilt', image: quilt },
  { name: 'NeoForge', text: 'Instance.Overall.Icon.NeoForgeFox', image: neoForge },
  { name: 'Cleanroom', text: 'Instance.Overall.Icon.Cleanroom', image: cleanroom },
]

/// 0 跟随版本类型，1 使用实例内的自定义图片，其余按下标选择内置图标。
export const autoLogo = '0'
export const customLogo = '1'
const firstBuiltinKey = 2

/// 实例列表等处的版本图标仍然使用这批图片。
export const instancePictures = {
  anvil,
  cleanroom,
  cobbleStone,
  commandBlock,
  egg,
  fabric,
  goldBlock,
  grassBlock,
  grassPath,
  labyMod,
  neoForge,
  quilt,
}

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

/// 选项编号对应的内置图片；自动与自定义没有图片。
export function imageFromSelection(selection: string) {
  return builtinLogos[Number(selection) - firstBuiltinKey]?.image ?? null
}
