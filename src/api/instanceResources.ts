import { invoke } from '@tauri-apps/api/core'

export type InstanceResourceKind =
  'mod' | 'resourcepack' | 'shader' | 'schematic' | 'screenshot' | 'save' | 'datapack'

export interface InstanceResourceEntry {
  name: string
  path: string
  /** File size in bytes; folder entries report 0. */
  size: number
  /** Seconds since the Unix epoch. */
  modified: number
  enabled: boolean
  icon?: string
  /** Opaque token for the `pclinstance` protocol, set for entries that have a picture. */
  token?: string
}

export function listInstanceResources(
  instanceDirectory: string,
  kind: InstanceResourceKind,
  save?: string,
) {
  return invoke<InstanceResourceEntry[]>('list_instance_resources', {
    instanceDirectory,
    kind,
    save,
  })
}

export function openInstanceResourceFolder(
  instanceDirectory: string,
  kind: InstanceResourceKind,
  save?: string,
) {
  return invoke<void>('open_instance_resource_folder', { instanceDirectory, kind, save })
}

export function openInstancePath(instanceDirectory: string, path: string) {
  return invoke<void>('open_instance_path', { instanceDirectory, path })
}

export function setInstanceResourcesEnabled(
  instanceDirectory: string,
  paths: string[],
  enabled: boolean,
) {
  return invoke<void>('set_instance_resources_enabled', { instanceDirectory, paths, enabled })
}

export function deleteInstanceResources(instanceDirectory: string, paths: string[]) {
  return invoke<void>('delete_instance_resources', { instanceDirectory, paths })
}

export function importInstanceResources(
  instanceDirectory: string,
  kind: InstanceResourceKind,
  sources: string[],
  save?: string,
) {
  return invoke<void>('import_instance_resources', { instanceDirectory, kind, sources, save })
}
