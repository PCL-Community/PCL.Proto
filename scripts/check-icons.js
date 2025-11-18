import fs from 'fs'
import path from 'path'
import { execSync } from 'child_process'

// 检查 src-tauri/icons 文件夹是否存在或为空
// 如果不存在或为空，则运行 tauri icon 命令生成图标
const iconsDir = path.join(process.cwd(), 'src-tauri', 'icons')

try {
  if (!fs.existsSync(iconsDir)) {
    console.log('Icons directory not found, generating icons...')
    execSync('npx tauri icon', { stdio: 'inherit' })
  } else {
    const files = fs.readdirSync(iconsDir)
    if (files.length === 0) {
      console.log('Icons directory is empty, generating icons...')
      execSync('npx tauri icon', { stdio: 'inherit' })
    }
  }
  process.exit(0)
} catch (err) {
  console.error('Error checking or generating icons:', err.message)
  process.exit(0)
}
