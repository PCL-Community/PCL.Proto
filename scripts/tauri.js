import path from 'node:path'
import { existsSync } from 'node:fs'

const args = process.argv.slice(2)
const env = { ...process.env }
if (process.platform === 'win32' && ['dev', 'build'].includes(args[0])) {
  const check = Bun.spawnSync([
    'powershell.exe', '-NoProfile', '-ExecutionPolicy', 'Bypass',
    '-File', 'scripts/windows/setup-development.ps1', '-CheckOnly',
  ], { stdout: 'inherit', stderr: 'inherit' })
  if (check.exitCode !== 0) process.exit(check.exitCode ?? 1)
  const tools = path.resolve('target/build-tools')
  if (!env.PROTOC || !existsSync(env.PROTOC)) {
    env.PROTOC = Bun.which('protoc') ?? path.join(tools, 'protoc-33.0', 'bin', 'protoc.exe')
  }
  env.WINDIVERT_DRIVER ??= path.join(tools, 'windivert-2.2.2', 'WinDivert-2.2.2-A', 'x64', 'WinDivert64.sys')
}
if (['dev', 'build'].includes(args[0])) {
  const icons = Bun.spawnSync(['bun', 'scripts/check-icons.js'], { stdout: 'inherit', stderr: 'inherit' })
  if (icons.exitCode !== 0) process.exit(icons.exitCode ?? 1)
}
const tauri = Bun.spawn(['bun', 'x', '--no-install', 'tauri', ...args], {
  env, stdin: 'inherit', stdout: 'inherit', stderr: 'inherit',
})
process.exit(await tauri.exited)
