//! CE 图片路径的唯一解析处：图片只在 `pcl/images` 存一份，按 CE 的名称定位。

/// CE 用静态方块图，命令方块保留一直在用的动画版本，因此不再有同名静态副本。
const animatedBlocks = new Set(['commandblock'])

/// 方块图标地址，`name` 为 CE 的 `images/Blocks` 文件名。
export function blockImage(name: string, base: string) {
  return `${base}pcl/images/Blocks/${name}.${animatedBlocks.has(name.toLowerCase()) ? 'gif' : 'png'}`
}

export function pclImagePath(source: string, base: string): string {
  if (/^(https?:|data:)/i.test(source)) return source
  const relative = source
    .replace(/^pack:\/\/application:,,,\//i, '')
    .replace(/^.*;component\//i, '')
    .replace(/^\/?images\//i, '')
  const block = /^blocks\/([^/.]+)\.png$/i.exec(relative)
  if (block) return blockImage(block[1], base)
  return `${base}pcl/images/${relative}`
}
