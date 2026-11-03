import assert from 'node:assert/strict'
import {
  renderMinecraftText,
  minecraftTextColor,
  minecraftTextStyle,
} from '../../src/util/minecraftText.ts'

// [text, color, bold, italic, underline, strikethrough]
const runs = (text, base) =>
  renderMinecraftText(text, base).map((run) => [
    run.text,
    run.color,
    run.bold,
    run.italic,
    run.underline,
    run.strikethrough,
  ])

// 重复渲染同一文本必须得到相同结果：旧实现把格式码累加进了共享的颜色表，
// 于是一次 §a§l 会让之后所有服务器的绿色文字都变粗。
assert.deepEqual(runs('§a§lgreen'), runs('§a§lgreen'))
assert.deepEqual(runs('§a§lgreen'), [['green', '#00FF00', true, undefined, undefined, undefined]])

// CE 语义：颜色码清除当前格式，格式码在同一颜色下累加，§r 全部重置。
assert.deepEqual(runs('§e§lY§nU'), [
  ['Y', '#FFFF00', true, undefined, undefined, undefined],
  ['U', '#FFFF00', true, undefined, true, undefined],
])
assert.deepEqual(runs('§lX§r§9Z'), [
  ['X', undefined, true, undefined, undefined, undefined],
  ['Z', '#0000FF', undefined, undefined, undefined, undefined],
])
assert.deepEqual(runs('plain'), [['plain', undefined, undefined, undefined, undefined, undefined]])
assert.deepEqual(runs(''), [])
assert.deepEqual(runs('ends with §'), [
  ['ends with §', undefined, undefined, undefined, undefined, undefined],
])
assert.deepEqual(runs('未知码 §z stays'), [
  ['未知码 §z stays', undefined, undefined, undefined, undefined, undefined],
])
// JSON description 的基底样式与 §代码共用一次解析。
assert.deepEqual(runs('A', { color: '#FFAA00', bold: true }), [
  ['A', '#FFAA00', true, undefined, undefined, undefined],
])

// 名字式颜色和 §代码取自同一张表。
assert.equal(minecraftTextColor('dark_aqua'), '#00AAAA')
assert.equal(minecraftTextColor('#123456'), '#123456')
assert.equal(minecraftTextColor('not_a_color'), undefined)
assert.equal(minecraftTextColor(undefined), undefined)
const style = minecraftTextStyle({ text: 'A', underline: true, strikethrough: true })
assert.equal(style.textDecoration, 'underline line-through')
assert.equal(style.color, undefined)
assert.equal(minecraftTextStyle({ text: 'A', italic: true }).fontStyle, 'italic')
console.log('Minecraft text codes, palette and idempotent rendering passed.')
