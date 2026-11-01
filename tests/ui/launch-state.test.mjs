import assert from 'node:assert/strict'
import fs from 'node:fs'
import { createPinia, setActivePinia } from 'pinia'
import { createServer } from 'vite'

const server = await createServer({
  server: { middlewareMode: true, hmr: false, watch: null },
  appType: 'custom',
})
const originalSetTimeout = globalThis.setTimeout
const originalClearTimeout = globalThis.clearTimeout
const timers = []
try {
  const launchPage = fs.readFileSync(
    new URL('../../src/views/Home/PageLaunchLeft.vue', import.meta.url),
    'utf8',
  )
  const transitions = fs.readFileSync(
    new URL('../../src/composables/usePageTransition.ts', import.meta.url),
    'utf8',
  )
  assert.match(launchPage, /transform: scale\(0\.96\)/)
  assert.match(launchPage, /transform: scale\(0\.8\)/)
  assert.doesNotMatch(launchPage, /scale\(0\.9\)/)
  assert.doesNotMatch(launchPage, /:not\(\.is-launching\)/)
  assert.match(transitions, /scale\(0\.95\)/)
  assert.doesNotMatch(transitions, /scale\(0\.9\)/)

  globalThis.setTimeout = (callback, delay) => {
    const timer = { callback, delay, cleared: false }
    timers.push(timer)
    return timer
  }
  globalThis.clearTimeout = (timer) => {
    if (timer) timer.cleared = true
  }

  const { useLaunchState } = await server.ssrLoadModule('/src/stores/launchState.ts')
  setActivePinia(createPinia())
  const state = useLaunchState()
  const attempt = state.start('Example instance', 'Offline')
  state.setPreparing(attempt, true)
  state.update(attempt, {
    stage: 'Minecraft.Launch.Stage.CompleteFiles',
    progress: 0.65,
    downloading: false,
  })

  const remountedHome = useLaunchState()
  assert.equal(remountedHome.launching?.name, 'Example instance')
  assert.equal(remountedHome.launching?.progress, 0.65)
  assert.equal(remountedHome.preparing, true)

  const cancelled = remountedHome.requestCancel()
  assert.equal(cancelled, attempt)
  assert.equal(remountedHome.isCancelled(attempt), true)
  remountedHome.returnToHome(attempt)
  assert.equal(remountedHome.leavingLaunch, true)
  assert.equal(remountedHome.launching?.progress, 0.65)
  const leaveTimer = timers.find((timer) => timer.delay === 360)
  assert.ok(leaveTimer)
  leaveTimer.callback()
  assert.equal(remountedHome.launching, null)
  assert.equal(remountedHome.leavingLaunch, false)
  console.log('Launch state survives Home remounts and clears after its return animation.')
} finally {
  globalThis.setTimeout = originalSetTimeout
  globalThis.clearTimeout = originalClearTimeout
  await server.close()
}
