import assert from 'node:assert/strict'
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
  console.log('Shared global and instance memory settings render correctly.')
} finally {
  await server.close()
}

console.log('Shared memory settings and button widgets render for global and instance scopes.')
