import { invoke } from '@tauri-apps/api/core'

export type IsolationDefault = 'off' | 'modded' | 'non_release' | 'modded_or_non_release' | 'all'

export const isolationChoices: IsolationDefault[] = [
  'off',
  'modded',
  'non_release',
  'modded_or_non_release',
  'all',
]
export const getIsolationDefault = () => invoke<IsolationDefault>('get_isolation_default')
export const setIsolationDefault = (value: IsolationDefault) =>
  invoke<void>('set_isolation_default', { value })
export const getInstanceIsolation = (instanceDirectory: string) =>
  invoke<boolean>('get_instance_isolation', { instanceDirectory })
export const setInstanceIsolation = (instanceDirectory: string, value: boolean) =>
  invoke<void>('set_instance_isolation', { instanceDirectory, value })
