import { invoke } from '@tauri-apps/api/core'
import { createSettingsPersistence } from '@/util/settingsPersistence'

export interface LaunchArguments {
  jvm: string
  game: string
  classpath: string
}
export const launchArgumentKeys = ['TextAdvanceJvm', 'TextAdvanceGame', 'TextAdvanceClasspathHead']
export const launchArgumentsStorage = createSettingsPersistence(
  (instanceDirectory) => invoke<LaunchArguments>('get_launch_arguments', { instanceDirectory }),
  (instanceDirectory, settings) =>
    invoke<LaunchArguments>('set_launch_arguments', { instanceDirectory, settings }),
)
