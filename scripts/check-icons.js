// src-tauri/icons is not committed; generate it from app-icon.png before Tauri reads the bundle config.
const iconsDir = 'src-tauri/icons'
let hasIcons
try {
  hasIcons = !new Bun.Glob('*').scanSync({ cwd: iconsDir }).next().done
} catch (error) {
  if (error.code !== 'ENOENT') throw error
  hasIcons = false
}

if (!hasIcons) {
  console.log('Launcher icons missing, running `tauri icon`...')
  const result = Bun.spawnSync(['bun', 'run', 'tauri', 'icon'], {
    stdout: 'inherit',
    stderr: 'inherit',
  })
  if (result.exitCode !== 0) process.exit(result.exitCode ?? 1)
}
