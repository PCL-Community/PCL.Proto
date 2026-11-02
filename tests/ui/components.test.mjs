import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { readFileSync } from 'node:fs'
import { createSSRApp, h } from 'vue'
import { renderToString } from 'vue/server-renderer'
import { createServer } from 'vite'

const server = await createServer({ server: { middlewareMode: true }, appType: 'custom' })
try {
  const { default: MemorySettings } = await server.ssrLoadModule(
    '/src/components/widget/PMemorySettings.vue',
  )
  const { default: Button } = await server.ssrLoadModule('/src/components/widget/PButton.vue')
  const renderButton = (props) => renderToString(createSSRApp({ render: () => h(Button, props) }))
  assert.match(await renderButton({ compact: true }), /class="[^"]*\bcompact\b/)
  assert.doesNotMatch(await renderButton({}), /class="[^"]*\bcompact\b/)
  const render = (props) => renderToString(createSSRApp({ render: () => h(MemorySettings, props) }))
  const global = await render({ java64: null })
  const instance = await render({ instance: true, directory: '/example', java64: true })
  assert.doesNotMatch(global, /data-control="RadioRamType2"/)
  assert.match(instance, /data-control="RadioRamType2"/)
  for (const html of [global, instance]) {
    assert.match(html, /data-control="SliderRamCustom"[^>]*disabled/)
    assert.match(html, /data-control="SliderRamInitialCustom"[^>]*disabled/)
    assert.match(html, /data-control="PanRamDisplay"/)
  }
  const source = await readFile(
    new URL('../../src/components/widget/PMemorySettings.vue', import.meta.url),
    'utf8',
  )
  assert.match(source, /\.bar \{\s+height: 4px;/)
  console.log('Shared global and instance memory settings render correctly.')
} finally {
  await server.close()
}

const read = (path) => readFileSync(path, 'utf8')

const base = read('src/assets/base.css')
const pcl = read('src/assets/pcl.css')

// 悬停/按下背景是主题色的半透明叠加：源色阶 ce-color-6/7 在深色主题下比表面更暗，会把悬停画成一块黑。
for (const [token, percent] of [
  ['--color-tint-lighter', 10],
  ['--half-transparent-blue', 22],
])
  assert.ok(
    base.includes(`${token}: color-mix(in srgb, var(--color-tint) ${percent}%, transparent);`),
    `${token} must stay a translucent tint veil, not a CE tone step`,
  )
assert.doesNotMatch(base, /--color-tint-lighter: var\(--ce-color-\d\)/)
assert.doesNotMatch(base, /--half-transparent-blue: var\(--ce-color-\d\)/)

// 静止态与悬停态同色，只有 alpha 不同。
assert.ok(
  base.includes('--color-tint-rest: rgb(from var(--color-tint) r g b / 0);'),
  '--color-tint-rest must be the tint at alpha 0',
)

const surfaces = [
  'src/components/widget/PButton.vue',
  'src/components/widget/PListItem.vue',
  'src/components/widget/PIconTextButton.vue',
  'src/components/widget/SideNavItem.vue',
  'src/components/widget/PTextBox.vue',
  'src/components/widget/PCompItem.vue',
]
for (const file of surfaces)
  assert.match(
    read(file),
    /background(-color)?: var\(--color-tint-rest\)/,
    `${file} animates a background and must rest on --color-tint-rest`,
  )
assert.match(pcl, /\.pcl-chip \{[\s\S]*?background: var\(--color-tint-rest\)/)
assert.match(pcl, /\.pcl-chip:active \{[\s\S]*?background: var\(--half-transparent-blue\)/)

// 背景动画统一由 CSS 承担，Motion 只负责缩放。
assert.doesNotMatch(
  read('src/components/widget/PButton.vue'),
  /backgroundColor/,
  'PButton must not animate colors through Motion',
)

// CE MyIconButton 禁用时不改色；Java 保存期间保留点击保护，图标仍用共用主题色。
const java = read('src/views/Setup/PageSetupJava.vue')
const disabledIcon = pcl.match(/\.pcl-icon-button:disabled\s*\{([^}]+)\}/)?.[1]
assert.ok(disabledIcon)
assert.doesNotMatch(disabledIcon, /color\s*:|opacity\s*:|filter\s*:/)
assert.match(disabledIcon, /cursor: not-allowed/)
assert.equal((java.match(/:disabled="saving \|\| loading"/g) || []).length, 2)

// Home 的三个操作与列表小图标使用同一个 hover 色，不再覆盖成最深主题色。
const profile = read('src/views/Home/PageLoginProfileSkin.vue')
assert.equal((profile.match(/class="pcl-icon-button profile-action"/g) || []).length, 3)
assert.doesNotMatch(profile, /color: var\(--color-tint-dark\)/)
const profileList = read('src/views/Home/PageLoginProfile.vue')
assert.match(profileList, /class="pcl-icon-button create-profile"/)
assert.doesNotMatch(profileList, /color: var\(--color-tint-dark\)/)

console.log('Hover/press overlays, Java busy icon colors and shared Home icon hover checked.')
