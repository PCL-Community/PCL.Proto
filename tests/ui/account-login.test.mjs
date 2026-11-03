import assert from 'node:assert/strict'
import fs from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { build } from 'esbuild'
import { createPinia, setActivePinia } from 'pinia'

const root = fileURLToPath(new URL('../../', import.meta.url))
const outfile = path.join(root, 'node_modules/.tmp/account-login.mjs')
await fs.mkdir(path.dirname(outfile), { recursive: true })
await build({
  stdin: {
    contents: "export { useAccountInfo, toAuthFailure } from './src/stores/account'",
    resolveDir: root,
    loader: 'ts',
  },
  outfile,
  bundle: true,
  platform: 'node',
  format: 'esm',
  external: ['vue', 'pinia'],
  define: { 'import.meta.env.BASE_URL': '"/"' },
  alias: { '@': path.join(root, 'src') },
  logLevel: 'silent',
  plugins: [
    {
      name: 'mocks',
      setup(builder) {
        builder.onResolve({ filter: /^@tauri-apps\/api\/core$/ }, (args) => ({
          path: args.path,
          namespace: 'mock',
        }))
        builder.onLoad({ filter: /.*/, namespace: 'mock' }, () => ({
          contents: `
            export const isTauri = () => true
            export const invoke = (name, args) => globalThis.testInvoke(name, args)
            export class Channel {}
          `,
        }))
      },
    },
  ],
})
const { useAccountInfo, toAuthFailure } = await import(pathToFileURL(outfile).href)

// 轮询用 setTimeout，测试里手动推进，避免真实等待。
const originalSetTimeout = globalThis.setTimeout
const originalClearTimeout = globalThis.clearTimeout
const timers = []
globalThis.setTimeout = (callback, delay) => {
  const timer = { callback, delay, cleared: false, ran: false }
  timers.push(timer)
  return timer
}
globalThis.clearTimeout = (timer) => {
  if (timer) timer.cleared = true
}

/** 等待 store 内部的异步链（刷新档案、拉皮肤）跑完。 */
const settle = () => new Promise((resolve) => originalSetTimeout(resolve, 20))

async function drain(rounds = 8) {
  for (let round = 0; round < rounds; round++) {
    const pending = timers.filter((timer) => !timer.cleared && !timer.ran)
    if (pending.length === 0) return
    for (const timer of pending) {
      timer.ran = true
      await timer.callback()
    }
    await Promise.resolve()
  }
}

const profile = {
  profileId: 'p1',
  userName: 'Player',
  uuid: 'a01e3843e5213998958af459800e4d11',
  tokenType: 'Bearer',
  profileType: 'microsoft',
  description: '',
  skinHeadId: '',
}

try {
  // ① 设备码登录成功后刷新档案列表与皮肤
  setActivePinia(createPinia())
  const calls = []
  let states = []
  globalThis.testInvoke = (name, args) => {
    calls.push([name, args])
    if (name === 'start_login') return Promise.resolve('session-1')
    if (name === 'login_state')
      return states.length
        ? Promise.resolve(states.shift())
        : Promise.reject({ code: 'session_not_found', message: 'gone' })
    if (name === 'get_profiles') return Promise.resolve({ profiles: [profile], current: 'p1' })
    if (name === 'fetch_profile_skin') return Promise.resolve({ skin: null, default: 'Alex' })
    return Promise.resolve()
  }
  const account = useAccountInfo()
  await account.startLogin({ profileType: 'microsoft' })
  assert.equal(account.loggingIn, true)
  assert.equal(account.loginState.status, 'working')

  states = [
    {
      status: 'deviceCode',
      userCode: 'CODE',
      verificationUri: 'https://microsoft.com/devicelogin',
      verificationUriComplete: 'https://microsoft.com/devicelogin?code=CODE',
      interval: 5,
    },
  ]
  await drain(1)
  assert.equal(account.loginState.status, 'deviceCode')
  assert.equal(account.loginState.userCode, 'CODE')
  assert.equal(account.loggingIn, true)

  states = [{ status: 'succeeded', profile }]
  await drain(3)
  await settle()
  assert.equal(account.loginState.status, 'succeeded')
  assert.equal(account.loggingIn, false)
  assert.deepEqual(account.profiles, [profile])
  assert.equal(account.currentId, 'p1')
  assert.equal(account.skinFor('p1'), '/pcl/images/Skins/Alex.png')
  assert.ok(calls.some(([name]) => name === 'get_profiles'))
  assert.ok(
    calls.some(
      ([name, args]) => name === 'fetch_profile_skin' && args.profileId === 'p1' && args.refresh,
    ),
  )

  // ② 取消登录会通知后端并停止轮询
  setActivePinia(createPinia())
  const cancelCalls = []
  globalThis.testInvoke = (name, args) => {
    cancelCalls.push([name, args])
    if (name === 'start_login') return Promise.resolve('session-2')
    if (name === 'login_state') return Promise.resolve({ status: 'working' })
    return Promise.resolve()
  }
  const cancelling = useAccountInfo()
  await cancelling.startLogin({
    profileType: 'authlib',
    server: 'https://littleskin.cn/api/yggdrasil',
    username: 'user',
    password: 'pass',
  })
  assert.deepEqual(cancelCalls[0], [
    'start_login',
    {
      request: {
        profileType: 'authlib',
        server: 'https://littleskin.cn/api/yggdrasil',
        username: 'user',
        password: 'pass',
      },
    },
  ])
  await cancelling.cancelLogin()
  assert.equal(cancelling.loginState.status, 'cancelled')
  assert.equal(cancelling.loggingIn, false)
  assert.ok(
    cancelCalls.some(([name, args]) => name === 'cancel_login' && args.sessionId === 'session-2'),
  )
  const timersBefore = timers.length
  await drain(2)
  assert.equal(timers.length, timersBefore, '取消后不再继续轮询')

  // ③ 登录失败保留后端错误码，交给界面映射弹窗
  setActivePinia(createPinia())
  globalThis.testInvoke = (name) => {
    if (name === 'start_login') return Promise.resolve('session-3')
    if (name === 'login_state')
      return Promise.resolve({ status: 'failed', code: 'xsts_2148916233', message: 'no xbox' })
    return Promise.resolve()
  }
  const failing = useAccountInfo()
  await failing.startLogin({ profileType: 'microsoft' })
  await drain(2)
  assert.equal(failing.loginState.status, 'failed')
  assert.equal(failing.loginState.code, 'xsts_2148916233')
  assert.equal(failing.loggingIn, false)

  // ④ 多角色时把选择回传给后端
  setActivePinia(createPinia())
  const selectCalls = []
  globalThis.testInvoke = (name, args) => {
    selectCalls.push([name, args])
    if (name === 'start_login') return Promise.resolve('session-4')
    if (name === 'login_state')
      return Promise.resolve({
        status: 'selectProfile',
        candidates: [
          { id: 'a', name: 'A' },
          { id: 'b', name: 'B' },
        ],
      })
    return Promise.resolve()
  }
  const selecting = useAccountInfo()
  await selecting.startLogin({ profileType: 'yggdrasilConnect' })
  await drain(1)
  await settle()
  assert.equal(selecting.loginState.status, 'selectProfile')
  await selecting.chooseCandidate('b')
  assert.ok(
    selectCalls.some(([name, args]) => name === 'select_login_profile' && args.candidateId === 'b'),
  )

  // ⑤ 取消选择档案时传 null（CE 的 Select(null)）
  setActivePinia(createPinia())
  const selectNull = []
  globalThis.testInvoke = (name, args) => {
    selectNull.push([name, args])
    if (name === 'get_profiles' || name === 'select_profile')
      return Promise.resolve({ profiles: [], current: null })
    if (name === 'fetch_profile_skin') return Promise.resolve({ skin: null, default: 'Steve' })
    return Promise.resolve()
  }
  const clearing = useAccountInfo()
  await clearing.select(null)
  assert.deepEqual(selectNull[0], ['select_profile', { profileId: null }])

  // ⑥ Tauri 的错误对象按 {code, message} 解析
  assert.deepEqual(toAuthFailure({ code: 'invalid_grant', message: 'expired' }), {
    code: 'invalid_grant',
    message: 'expired',
  })
  assert.deepEqual(toAuthFailure('boom'), { code: 'unknown', message: 'boom' })

  console.log('Account login sessions, cancellation, profile selection and skins passed.')
} finally {
  globalThis.setTimeout = originalSetTimeout
  globalThis.clearTimeout = originalClearTimeout
  delete globalThis.testInvoke
  await fs.rm(outfile, { force: true })
}
