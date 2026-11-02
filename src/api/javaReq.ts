import { Channel, invoke } from '@tauri-apps/api/core'

export type JavaSource = 'auto_scanned' | 'auto_installed' | 'manual_added'

/** Flat IPC view of `minecraft::java::JavaEntry`. */
export interface IJavaRuntimeInfo {
  java_folder: string
  java_exe: string
  version: string
  /** Raw `IMPLEMENTOR` from the JDK; empty when it could not be read. */
  vendor: string
  architecture: 'X86' | 'X64' | 'Arm64' | 'FatFile' | 'Unknown'
  is_64_bit: boolean
  is_jre: boolean
  compatibility: 'Perfect' | 'Translation' | 'No' | 'Unknown'
  is_enabled: boolean
  source: JavaSource
}

/** Java 8 is reported as `1.8.0_x`, so normalize it for display/sorting. */
export function javaMajorVersion(version: string): number {
  const parts = version.split(/[._+-]/).map((part) => Number.parseInt(part, 10))
  const major = parts[0] ?? 0
  const minor = parts[1] ?? 0
  if (Number.isNaN(major)) return 0
  return major === 1 && minor > 0 ? minor : major
}

export const javaDisplayName = (runtime: IJavaRuntimeInfo) =>
  runtime.version
    ? `${runtime.is_jre ? 'JRE' : 'JDK'} ${javaMajorVersion(runtime.version)} (${runtime.version})`
    : `${runtime.is_jre ? 'JRE' : 'JDK'} (${runtime.vendor || '?'})`
export type JavaPreference =
  | { kind: 'global' | 'auto' }
  | { kind: 'exist'; java_exe: string }
  | { kind: 'relative'; relative_path: string }
export interface InstanceJavaSettings {
  preference: JavaPreference
  ignore_compatibility: boolean
}
export interface JavaSettingsInfo {
  global_java: string | null
  instance: InstanceJavaSettings
  has_instance_settings: boolean
  global_is_64_bit: boolean
  instance_is_64_bit: boolean
}
export interface JavaLaunchInfo {
  instance_directory: string
  requirements: {
    min: number[]
    max: number[]
    compatible_majors: number[]
    component: string | null
  }
  runtime: IJavaRuntimeInfo | null
  warnings: { key: string; version: string; min: string; max: string }[]
  download_major: number | null
}
export interface JavaInstallProgress {
  completed: number
  total: number
}
export const getJavaList = () => invoke<IJavaRuntimeInfo[]>('get_java_list')
export const refreshJavaList = () => invoke<IJavaRuntimeInfo[]>('refresh_java_list')
export const getJavaSettings = (instanceDirectory?: string) =>
  invoke<JavaSettingsInfo>('get_java_settings', { instanceDirectory })
export const setGlobalJava = (javaExe: string | null) => invoke('set_global_java', { javaExe })
export const setJavaEnabled = (javaExe: string, enabled: boolean) =>
  invoke('set_java_enabled', { javaExe, enabled })
export const setInstanceJavaSettings = (
  instanceDirectory: string,
  settings: InstanceJavaSettings,
) => invoke('set_instance_java_settings', { instanceDirectory, settings })
export const selectRelativeJava = () => invoke<JavaPreference | null>('select_relative_java')
export const prepareGameJava = (instanceDirectory: string) =>
  invoke<JavaLaunchInfo>('prepare_game_java', { instanceDirectory })
export function downloadGameJava(
  instanceDirectory: string,
  onProgress: (progress: JavaInstallProgress) => void,
) {
  const progress = new Channel<JavaInstallProgress>()
  progress.onmessage = onProgress
  return invoke<IJavaRuntimeInfo>('download_game_java', {
    instanceDirectory,
    progress,
  })
}
