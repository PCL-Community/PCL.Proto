import { invoke } from '@tauri-apps/api/core'

export interface LaunchArguments {
  jvm: string
  game: string
  classpath: string
}
export const launchArgumentKeys = ['TextAdvanceJvm', 'TextAdvanceGame', 'TextAdvanceClasspathHead']
const queues = new Map<string, Promise<unknown>>()
const drafts = new Map<string, LaunchArguments>()
const errors = new Map<string, string>()
const revisions = new Map<string, number>()
const keyFor = (directory: string | null) =>
  directory === null ? 'global' : `instance:${directory}`

export function stageLaunchArguments(directory: string | null, settings: LaunchArguments) {
  const key = keyFor(directory)
  drafts.set(key, { ...settings })
  revisions.set(key, (revisions.get(key) || 0) + 1)
  errors.delete(key)
}

export async function readLaunchArguments(directory: string | null) {
  const key = keyFor(directory)
  await queues.get(key)
  const settings =
    drafts.get(key) ||
    (await invoke<LaunchArguments>('get_launch_arguments', { instanceDirectory: directory }))
  return { settings: { ...settings }, error: errors.get(key) || '', dirty: drafts.has(key) }
}

export function saveLaunchArguments(directory: string | null) {
  const key = keyFor(directory)
  const settings = drafts.get(key)
  if (!settings) return Promise.resolve()
  const revision = revisions.get(key)
  const result = (queues.get(key) || Promise.resolve())
    .then(() =>
      invoke<LaunchArguments>('set_launch_arguments', { instanceDirectory: directory, settings }),
    )
    .then(() => {
      if (revision === revisions.get(key)) {
        drafts.delete(key)
        errors.delete(key)
      }
    })
    .catch((error) => {
      if (revision === revisions.get(key)) errors.set(key, String(error))
      throw error
    })
  queues.set(
    key,
    result.catch(() => {}),
  )
  return result
}
