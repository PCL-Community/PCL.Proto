import { mkdir, mkdtemp, copyFile, chmod, writeFile } from 'node:fs/promises'
import path from 'node:path'

if (process.platform === 'win32') {
  const result = Bun.spawnSync([
    'powershell.exe', '-NoProfile', '-ExecutionPolicy', 'Bypass',
    '-File', 'scripts/windows/package-portable.ps1',
  ], { stdout: 'inherit', stderr: 'inherit' })
  process.exit(result.exitCode ?? 1)
}
if (process.platform !== 'linux') throw new Error('Use the .app bundle on macOS.')
const { version } = await Bun.file('package.json').json()
const arch = { x64: 'x64', arm64: 'arm64' }[process.arch]
if (!arch) throw new Error(`Unsupported architecture: ${process.arch}`)
const directory = path.resolve(`target/portable/PCL.Proto_${version}_linux_${arch}`)
await mkdir(path.dirname(directory), { recursive: true })
const stagingRoot = await mkdtemp(path.join(path.dirname(directory), '.staging-'))
const staging = path.join(stagingRoot, path.basename(directory))
await mkdir(staging)
await copyFile(path.join(process.env.CARGO_TARGET_DIR ?? 'target', 'release/pcl-proto'), path.join(staging, 'pcl-proto'))
await chmod(path.join(staging, 'pcl-proto'), 0o755)
await writeFile(path.join(staging, 'README.txt'), `PCL.Proto Linux ${arch}\n\nExtract the whole directory and run ./pcl-proto.\nThis archive uses system libraries, including GTK 3 and WebKitGTK 4.1.\nOn Ubuntu 22.04: sudo apt install libwebkit2gtk-4.1-0 libayatana-appindicator3-1\nFor automatic dependency installation use the .deb or .rpm package.\n`)
const result = Bun.spawnSync(['tar', '-czf', `${directory}.tar.gz`, '-C', stagingRoot, path.basename(directory)], {
  stdout: 'inherit', stderr: 'inherit',
})
process.exit(result.exitCode ?? 1)
