import assert from 'node:assert/strict'
import fs from 'node:fs/promises'
import { createServer } from 'vite'

const server = await createServer({
  server: { middlewareMode: true, hmr: false, watch: null },
  appType: 'custom',
})
const originals = Object.fromEntries(
  ['window', 'document', 'Element', 'isTauri'].map((key) => [
    key,
    Object.getOwnPropertyDescriptor(globalThis, key),
  ]),
)
const calls = []
const listeners = new Map()
const originalConsoleError = console.error
const errors = []
class Element {
  constructor(anchor) {
    this.anchor = anchor
  }
  closest(selector) {
    assert.equal(selector, 'a[href]')
    return this.anchor
  }
}
try {
  console.error = (...args) => errors.push(args)
  globalThis.Element = Element
  globalThis.isTauri = true
  globalThis.window = {
    location: new URL('http://tauri.localhost/index.html'),
    __TAURI_INTERNALS__: {
      invoke: async (command, args) => calls.push([command, args]),
    },
    open: (...args) => calls.push(args),
  }
  globalThis.document = {
    addEventListener: (type, handler, capture) => {
      assert.equal(capture, true)
      listeners.set(type, handler)
    },
    removeEventListener: (type, handler, capture) => {
      assert.equal(capture, true)
      assert.equal(listeners.get(type), handler)
      listeners.delete(type)
    },
  }
  const { externalUrl, openExternal, installExternalLinkHandler } = await server.ssrLoadModule(
    '/src/util/externalLink.ts',
  )
  const { default: sideTip } = await server.ssrLoadModule('/src/composables/sideTip.ts')
  const warnings = []
  sideTip.show = (...args) => warnings.push(args)

  for (const origin of ['http://localhost:5173', 'http://tauri.localhost', 'tauri://localhost']) {
    window.location = new URL(`${origin}/index.html`)
    assert.equal(externalUrl('https://example.com/a?b=c#d'), 'https://example.com/a?b=c#d')
    for (const url of [
      '/download',
      '#news',
      `${origin}/setup`,
      'javascript:alert(1)',
      'file:///tmp/a',
      'mailto:a@example.com',
    ]) {
      assert.equal(externalUrl(url), undefined, url)
      assert.equal(await openExternal(url), false, url)
    }
  }
  window.location = new URL('http://tauri.localhost/index.html')
  for (const value of [null, undefined, '']) {
    assert.equal(externalUrl(value), undefined)
    assert.equal(await openExternal(value), false)
  }
  assert.equal(externalUrl('//example.com/path'), 'http://example.com/path')
  assert.equal(await openExternal('https://example.com'), true)
  assert.deepEqual(calls.pop(), ['open_external_url', { url: 'https://example.com/' }])
  assert.equal(await openExternal('https://user:pass@example.com'), false)
  window.__TAURI_INTERNALS__.invoke = async () => {
    throw new Error('No browser')
  }
  assert.equal(await openExternal('https://example.com'), false)
  assert.equal(warnings.length, 2)
  assert.equal(errors.length, 2)
  window.__TAURI_INTERNALS__.invoke = async (command, args) => calls.push([command, args])

  const dispose = installExternalLinkHandler()
  function click(type, button, href, textTarget = false, defaultPrevented = false) {
    const element = new Element({ href })
    const event = {
      type,
      button,
      defaultPrevented,
      target: textTarget ? { parentElement: element } : element,
      stopped: false,
      preventDefault() {
        this.defaultPrevented = true
      },
      stopPropagation() {
        this.stopped = true
      },
    }
    listeners.get(type)(event)
    return event
  }
  for (const [type, button, textTarget] of [
    ['click', 0, false],
    ['auxclick', 1, true],
  ]) {
    const event = click(type, button, 'https://example.com/', textTarget)
    assert.equal(event.defaultPrevented, true)
    assert.equal(event.stopped, true)
    assert.equal(calls.pop()[0], 'open_external_url')
  }
  click('auxclick', 2, 'https://example.com/')
  click('click', 0, 'https://example.com/', false, true)
  assert.equal(click('click', 0, 'http://tauri.localhost/setup').defaultPrevented, false)
  assert.equal(calls.length, 0)
  dispose()
  assert.equal(listeners.size, 0)

  globalThis.isTauri = false
  installExternalLinkHandler()()
  assert.equal(listeners.size, 0)
  assert.equal(await openExternal('https://example.com'), true)
  assert.deepEqual(calls.pop(), ['https://example.com/', '_blank', 'noopener,noreferrer'])

  // Keep imperative callers on the shared entry point as new pages are added.
  async function checkCallers(directory) {
    for (const item of await fs.readdir(directory, { withFileTypes: true })) {
      const path = new URL(`${item.name}${item.isDirectory() ? '/' : ''}`, directory)
      if (item.isDirectory()) await checkCallers(path)
      else if (
        /\.(ts|tsx|vue)$/.test(item.name) &&
        path.pathname.endsWith('/util/externalLink.ts') === false
      ) {
        assert.doesNotMatch(await fs.readFile(path, 'utf8'), /window\.open\s*\(/, path.pathname)
      }
    }
  }
  await checkCallers(new URL('../../src/', import.meta.url))
  console.log('External URL validation, native/browser opening, click handling and cleanup passed.')
} finally {
  console.error = originalConsoleError
  for (const [key, descriptor] of Object.entries(originals)) {
    if (descriptor) Object.defineProperty(globalThis, key, descriptor)
    else delete globalThis[key]
  }
  await server.close()
}
