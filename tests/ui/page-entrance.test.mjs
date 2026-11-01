import assert from 'node:assert/strict'
import { createServer } from 'vite'

const server = await createServer({
  server: { middlewareMode: true, hmr: false, watch: null },
  appType: 'custom',
  ssr: { noExternal: ['motion-v'] },
  optimizeDeps: { noDiscovery: true, include: [] },
  plugins: [
    {
      name: 'capture-entrance-animation',
      enforce: 'pre',
      resolveId(id) {
        if (id === 'motion-v') return '\0entrance-motion'
      },
      load(id) {
        if (id === '\0entrance-motion')
          return 'export const animate = (...args) => globalThis.entranceCalls.push(args)'
      },
    },
  ],
})
const saved = Object.fromEntries(
  ['window', 'Element', 'MutationObserver', 'entranceCalls'].map((key) => [
    key,
    Object.getOwnPropertyDescriptor(globalThis, key),
  ]),
)
try {
  let mutation
  let disconnected = false
  let reduced = false
  globalThis.entranceCalls = []
  globalThis.window = { matchMedia: () => ({ matches: reduced }) }
  globalThis.Element = class {
    constructor(card = false, top = 0) {
      this.card = card
      this.top = top
      this.children = []
      this.style = {}
      this.nodeType = 1
    }
    matches() {
      return this.card
    }
    querySelector() {
      return this.children.some((child) => child.card)
        ? this.children.find((child) => child.card)
        : null
    }
    closest() {
      return this.card ? this : null
    }
    getBoundingClientRect() {
      return { top: this.top, bottom: this.top + 40, height: 40 }
    }
  }
  globalThis.MutationObserver = class {
    constructor(callback) {
      mutation = callback
    }
    observe() {}
    disconnect() {
      disconnected = true
    }
  }
  const { vPageEntrance, refreshPageEntrance } = await server.ssrLoadModule(
    '/src/util/cardDropAnimate.ts',
  )
  const content = new Element()
  const loader = new Element(true)
  content.children = [loader]
  const root = new Element()
  root.querySelector = () => content
  const flush = () => new Promise((resolve) => queueMicrotask(resolve))
  vPageEntrance.mounted(root)
  assert.deepEqual(entranceCalls[0][0], [loader])
  const first = new Element(true)
  const second = new Element(true)
  const offscreen = new Element(true, 500)
  content.children = [first, second, offscreen]
  mutation([{ target: content, addedNodes: content.children }])
  await flush()
  assert.deepEqual(
    entranceCalls[1][0],
    [first, second],
    'async results animate after replacing the loader',
  )
  mutation([{ target: first, addedNodes: [new Element()] }])
  await flush()
  assert.equal(
    entranceCalls.length,
    2,
    'expansion and inner updates do not replay the card entrance',
  )
  mutation([{ target: content, addedNodes: [first] }])
  await flush()
  assert.equal(entranceCalls.length, 2, 'existing cards are never animated twice')
  content.style.pointerEvents = 'none'
  vPageEntrance.updated(root, { value: '/b', oldValue: '/a' })
  await flush()
  assert.equal(entranceCalls.length, 2, 'entrance waits while the reused page is exiting')
  content.style.pointerEvents = ''
  refreshPageEntrance(root)
  await flush()
  assert.equal(entranceCalls.length, 3, 'reused route components replay their entrance')
  reduced = true
  vPageEntrance.updated(root, { value: '/c', oldValue: '/b' })
  await flush()
  assert.equal(entranceCalls.length, 3, 'reduced motion disables entrances')
  reduced = false
  mutation([{ target: content, addedNodes: [new Element()] }])
  vPageEntrance.unmounted(root)
  await flush()
  assert.ok(disconnected)
  assert.equal(entranceCalls.length, 3, 'queued work stops on unmount')
  console.log(
    'Unified synchronous/async entrances, reuse, no replay, reduced motion and cleanup passed.',
  )
} finally {
  for (const [key, descriptor] of Object.entries(saved)) {
    if (descriptor) Object.defineProperty(globalThis, key, descriptor)
    else delete globalThis[key]
  }
  await server.close()
}
