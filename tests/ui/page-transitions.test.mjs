import assert from 'node:assert/strict'
import { createServer } from 'vite'
import { createSSRApp, effectScope, nextTick, shallowRef } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'

const server = await createServer({
  server: { middlewareMode: true, hmr: false, watch: null },
  appType: 'custom',
})
const scope = effectScope()
try {
  const { usePageTransition } = await server.ssrLoadModule('/src/composables/usePageTransition.ts')
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      ...['a', 'b', 'c'].map((path) => ({ path: `/${path}`, component: {} })),
      {
        path: '/group',
        component: {},
        children: [
          { path: 'one', component: {} },
          { path: 'two', component: {} },
        ],
      },
    ],
  })
  const app = createSSRApp({})
  app.use(router)
  await router.push('/a')
  const animations = []
  const styleOperations = []
  const timers = []
  let reducedMotion = false
  let expectedDelay = 110
  globalThis.window = {
    matchMedia: () => ({ matches: reducedMotion }),
    setTimeout: (callback, delay) => {
      assert.equal(delay, expectedDelay)
      timers.push(callback)
    },
  }
  globalThis.getComputedStyle = () => {
    styleOperations.push('read')
    return { opacity: '1', transform: 'none' }
  }
  const block = () => ({
    children: [],
    getBoundingClientRect: () => ({ top: 0, bottom: 40, height: 40 }),
    matches: () => true,
    animate: (frames, options) => {
      styleOperations.push('write')
      const animation = {
        frames,
        options,
        cancelled: false,
        cancel() {
          this.cancelled = true
        },
      }
      animations.push(animation)
      return animation
    },
  })
  const content = {
    style: { pointerEvents: '' },
    children: [block(), block(), block()],
    getBoundingClientRect: () => ({ top: 0, bottom: 400, height: 400 }),
  }
  const page = shallowRef({
    querySelector: (selector) => (selector.includes('> aside') ? null : content),
  })
  app.runWithContext(() => scope.run(() => usePageTransition(page)))
  const flush = () => new Promise((resolve) => setImmediate(resolve))
  const finish = () => timers.splice(0).forEach((callback) => callback())

  const first = router.push('/b')
  await flush()
  assert.equal(router.currentRoute.value.path, '/a', 'keep outgoing content until exit finishes')
  assert.equal(content.style.pointerEvents, 'none')
  assert.deepEqual(
    styleOperations,
    ['read', 'read', 'read', 'write', 'write', 'write'],
    'read styles in one batch before animation writes',
  )
  assert.deepEqual(
    animations.map(({ options }) => options.delay),
    [0, 15, 30],
  )
  assert.ok(
    animations.every(
      ({ options, frames }) =>
        options.duration === 70 &&
        frames[1].opacity === 0 &&
        frames[1].transform.trim() === 'translateY(-6px)',
    ),
  )
  const second = router.push('/c')
  await flush()
  assert.equal(animations.length, 3, 'rapid navigation shares one exit')
  finish()
  await Promise.all([first, second])
  await nextTick()
  assert.equal(router.currentRoute.value.path, '/c')
  assert.equal(content.style.pointerEvents, '')
  assert.ok(animations.every(({ cancelled }) => cancelled))

  const pending = router.push('/b')
  await flush()
  await router.push('/c')
  await nextTick()
  assert.equal(content.style.pointerEvents, '', 'duplicate navigation restores the current page')
  finish()
  await pending

  const removeAbort = router.beforeResolve(() => false)
  const aborted = router.push('/b')
  await flush()
  finish()
  await aborted
  await nextTick()
  assert.equal(router.currentRoute.value.path, '/c')
  assert.equal(content.style.pointerEvents, '', 'aborted navigation restores interaction')
  assert.ok(animations.every(({ cancelled }) => cancelled))
  removeAbort()

  const count = animations.length
  await router.push('/c?filter=test')
  reducedMotion = true
  await router.push('/a')
  assert.equal(animations.length, count, 'query updates and reduced motion skip the exit')
  reducedMotion = false

  const standalone = { ...block(), style: { pointerEvents: '' } }
  page.value = {
    querySelector: (selector) =>
      selector.includes('> aside') || selector.includes('.subview') ? null : standalone,
  }
  const fullPage = router.push('/b')
  await flush()
  assert.equal(animations.length, count + 1, 'standalone content such as the editor also exits')
  finish()
  await fullPage
  await nextTick()
  assert.equal(standalone.style.pointerEvents, '')

  const enterGroup = router.push('/group/one')
  await flush()
  finish()
  await enterGroup
  expectedDelay = 130
  const subpage = router.push('/group/two')
  await flush()
  finish()
  await subpage
  content.children = Array.from({ length: 100 }, block)
  content.children.push({
    ...block(),
    getBoundingClientRect: () => ({ top: 500, bottom: 540, height: 40 }),
  })
  page.value = { querySelector: (selector) => (selector.includes('> aside') ? null : content) }
  const manyCards = router.push('/group/one')
  const beforeExit = animations.length
  await flush()
  assert.equal(animations.length - beforeExit, 100, 'offscreen cards do not start exit animations')
  assert.ok(
    animations.slice(-100).every(({ options }) => options.delay + options.duration <= 130),
    'all exit animations fit inside the navigation delay regardless of list length',
  )
  finish()
  await manyCards
  expectedDelay = 110
  for (const launch of [false, true]) {
    const aside = { ...block(), classList: { contains: () => launch } }
    page.value = { querySelector: (selector) => (selector.includes('> aside') ? aside : content) }
    const offset = animations.length
    const leaving = router.push(launch ? '/b' : '/a')
    await flush()
    const current = animations.slice(offset)
    assert.equal(current.length, 101, 'main-tab navigation also exits the sidebar')
    assert.equal(
      current[0].options.delay,
      current.at(-1).options.delay,
      'left and right start together',
    )
    assert.ok(
      current.at(-1).frames[1].transform.includes(launch ? 'scale(0.95)' : 'translateX(-6px)'),
    )
    finish()
    await leaving
    assert.ok(current.every((animation) => animation.cancelled))
  }
  console.log(
    'Page exit timing, rapid/duplicate/aborted navigation, full pages and reduced motion passed.',
  )
} finally {
  scope.stop()
  delete globalThis.window
  delete globalThis.getComputedStyle
  await server.close()
}
