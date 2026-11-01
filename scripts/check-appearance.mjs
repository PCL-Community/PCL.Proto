import assert from 'node:assert/strict'
import { build } from 'esbuild'
import 'vue'

const properties = new Map()
const classes = new Set()
let systemChange
globalThis.document = {
  documentElement: {
    style: { setProperty: (key, value) => properties.set(key, value) },
    classList: { toggle: (key, enabled) => (enabled ? classes.add(key) : classes.delete(key)) },
  },
}
const media = {
  matches: false,
  addEventListener: (_event, handler) => {
    systemChange = handler
  },
}
globalThis.window = { matchMedia: () => media }

const result = await build({
  entryPoints: ['src/api/appearance.ts'],
  bundle: true,
  write: false,
  format: 'esm',
  platform: 'node',
  plugins: [
    {
      name: 'settings',
      setup(builder) {
        builder.onResolve({ filter: /^vue$/ }, () => ({
          path: import.meta.resolve('vue'),
          external: true,
        }))
        builder.onResolve({ filter: /^@\/composables\/usePageSettings$/ }, () => ({
          path: 'settings',
          namespace: 'test',
        }))
        builder.onLoad({ filter: /.*/, namespace: 'test' }, () => ({
          contents:
            "import { reactive } from 'vue'; export const usePageSettings = (_, defaults) => ({ values: reactive(defaults) })",
          resolveDir: process.cwd(),
        }))
      },
    },
  ],
})
const { appearance } = await import(
  `data:text/javascript;base64,${Buffer.from(result.outputFiles[0].text).toString('base64')}`
)
assert.equal(classes.has('dark'), false)
const cat = properties.get('--ce-color-3')
assert.equal(cat, 'oklch(0.575 0.1704 255)')
assert.match(properties.get('--base-background'), /0\.84 0\.093 240/)
appearance.values.CheckBackgroundColorful = false
assert.equal(properties.get('--base-background'), 'var(--color-background)')
appearance.values.CheckBackgroundColorful = true
appearance.values.ComboLightColor = '0'
assert.notEqual(properties.get('--ce-color-3'), cat)
assert.match(properties.get('--base-background'), /0\.84 0\.093 220/)
appearance.values.ComboUiFont = 'Example Font'
assert.match(properties.get('--custom-font'), /Example Font/)
appearance.values.ComboUiMotdFont = 'MOTD Font'
assert.match(properties.get('--motd-font'), /MOTD Font/)
media.matches = true
systemChange()
assert.equal(classes.has('dark'), true)
assert.match(properties.get('--base-background'), /0\.168 0\.093 240/)
appearance.values.ComboDarkMode = '0'
assert.equal(classes.has('dark'), false)
appearance.values.ComboDarkMode = '1'
assert.equal(classes.has('dark'), true)
console.log('Appearance modes, colors, fonts and system changes passed.')
