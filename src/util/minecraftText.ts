// Minecraft 文本样式解析，取值与状态机语义对齐 PCL2-CE 的 MotdRenderer。
// 服务器卡片是深色石头背景，因此使用 CE 的暗色背景调色板。
const motdColors: Record<string, string> = {
  '0': '#000000',
  '1': '#0000AA',
  '2': '#00AA00',
  '3': '#00AAAA',
  '4': '#AA0000',
  '5': '#AA00AA',
  '6': '#FFAA00',
  '7': '#D3D3D3',
  '8': '#A9A9A9',
  '9': '#0000FF',
  a: '#00FF00',
  b: '#00FFFF',
  c: '#FF0000',
  d: '#FF00FF',
  e: '#FFFF00',
  f: '#FFFFFF',
}
// JSON description 里的名字式颜色与 §代码共用同一张表，避免出现两套配色。
const motdColorNames: Record<string, string> = {
  black: '0',
  dark_blue: '1',
  dark_aqua: '3',
  dark_red: '4',
  dark_purple: '5',
  gold: '6',
  gray: '7',
  dark_gray: '8',
  blue: '9',
  green: 'a',
  aqua: 'b',
  red: 'c',
  light_purple: 'd',
  yellow: 'e',
  white: 'f',
}
type MotdStyle = {
  color?: string
  bold?: boolean
  italic?: boolean
  underline?: boolean
  strikethrough?: boolean
}
type MotdRun = MotdStyle & { text: string }
// §k 混淆字符 CE 用 20ms 定时器滚动，这里按原样输出文本，未实现该动效。
export function renderMinecraftText(text: string, base: MotdStyle = {}): MotdRun[] {
  const runs: MotdRun[] = []
  let { color, bold, italic, underline, strikethrough } = base
  let buffer = ''
  const flush = () => {
    if (!buffer) return
    runs.push({ color, bold, italic, underline, strikethrough, text: buffer })
    buffer = ''
  }
  for (let i = 0; i < text.length; i++) {
    const char = text[i]
    if (char !== '\u00a7' || i === text.length - 1) {
      buffer += char
      continue
    }
    const code = text[i + 1].toLowerCase()
    // 未知样式码按原样保留文本，不改动当前样式，也不截断文本段。
    if (!(code in motdColors) && !'lrondm'.includes(code)) {
      buffer += char
      continue
    }
    i++
    flush()
    if (code === 'r') {
      color = undefined
      bold = italic = underline = strikethrough = undefined
    } else if (code in motdColors) {
      // CE 语义：颜色码清除当前格式，只保留颜色
      color = motdColors[code]
      bold = italic = underline = strikethrough = undefined
    } else if (code === 'l') bold = true
    else if (code === 'o') italic = true
    else if (code === 'n') underline = true
    else if (code === 'm') strikethrough = true
  }
  flush()
  return runs
}
export function minecraftTextStyle(run: MotdRun) {
  const decoration = [run.underline && 'underline', run.strikethrough && 'line-through'].filter(
    Boolean,
  )
  return {
    fontSize: '12px',
    color: run.color,
    fontWeight: run.bold ? 'bold' : undefined,
    fontStyle: run.italic ? 'italic' : undefined,
    textDecoration: decoration.length ? decoration.join(' ') : undefined,
  }
}
export function minecraftTextColor(name?: string) {
  if (!name) return undefined
  return name.startsWith('#') ? name : motdColors[motdColorNames[name]]
}
