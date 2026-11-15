const STORAGE_KEY = 'PCL.GameUpdateNotice'

/** 游戏管理页的「游戏更新提示」开关，默认开启以保持启动时检查更新的行为。 */
export function isGameUpdateNoticeEnabled(): boolean {
  try {
    return localStorage.getItem(STORAGE_KEY) !== 'false'
  } catch {
    return true
  }
}

export function setGameUpdateNoticeEnabled(enabled: boolean) {
  try {
    localStorage.setItem(STORAGE_KEY, String(enabled))
  } catch {
    // 存储不可用时保持当前会话行为即可。
  }
}
