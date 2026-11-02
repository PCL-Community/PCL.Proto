import fs from 'node:fs'
import { execSync } from 'node:child_process'

// src-tauri/icons is not committed; generate it from app-icon.png before Tauri reads the bundle config.
const iconsDir = 'src-tauri/icons'
if (!fs.existsSync(iconsDir) || fs.readdirSync(iconsDir).length === 0) {
  console.log('Launcher icons missing, running `tauri icon`...')
  execSync('npx tauri icon', { stdio: 'inherit' })
}
