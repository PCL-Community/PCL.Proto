// releases/latest 只提供正式版；开发版在相同基础版本的正式版之前。
export function isNewerRelease(release: string, current: string): boolean {
  const stable = /^v?(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:\+[\da-zA-Z.-]+)?$/
  const version = /^v?(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(-[\da-zA-Z.-]+)?(?:\+[\da-zA-Z.-]+)?$/
  const latest = stable.exec(release)
  const installed = version.exec(current)
  if (!latest || !installed) return false

  for (let index = 1; index <= 3; index++) {
    const difference = Number(latest[index]) - Number(installed[index])
    if (difference !== 0) return difference > 0
  }
  return Boolean(installed[4])
}
