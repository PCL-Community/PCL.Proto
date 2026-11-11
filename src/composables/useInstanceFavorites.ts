import { computed } from 'vue'
import {
  instanceMetadata,
  loadInstanceMetadata,
  updateInstanceMetadata,
} from '@/api/instanceMetadata'

const favorites = computed(() =>
  Object.entries(instanceMetadata)
    .filter(([, value]) => value.starred)
    .map(([directory]) => directory),
)
const pending = new Map<string, Promise<void>>()
export function useInstanceFavorites() {
  function isFavorite(directory: string) {
    return !!instanceMetadata[directory]?.starred
  }
  function toggleFavorite(directory: string) {
    const task = (pending.get(directory) || Promise.resolve()).then(async () => {
      if (!instanceMetadata[directory]) await loadInstanceMetadata([directory])
      await updateInstanceMetadata(directory, { starred: !isFavorite(directory) })
    })
    pending.set(
      directory,
      task.catch(() => {}),
    )
    return task
  }
  return { favorites, isFavorite, toggleFavorite }
}
