import { pclText } from '@/api/pclLocale'
import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ISearchHit } from '@/types/comp'
import sideTip from '@/composables/sideTip'
const key = 'pcl-comp-favorites'
export const useCompFavorites = defineStore('comp-favorites', () => {
  const items = ref<ISearchHit[]>([])
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(key) || '[]')
    if (Array.isArray(saved))
      items.value = saved.filter(
        (item) =>
          item &&
          ['project_id', 'title', 'description', 'icon_url', 'date_modified'].every(
            (field) => typeof item[field] === 'string',
          ) &&
          typeof item.downloads === 'number' &&
          Array.isArray(item.categories) &&
          item.categories.every((value: unknown) => typeof value === 'string') &&
          ['versions', 'loaders'].every(
            (field) =>
              item[field] === undefined ||
              (Array.isArray(item[field]) &&
                item[field].every((value: unknown) => typeof value === 'string')),
          ),
      )
  } catch {
    /* Invalid local preferences must not prevent opening the resource pages. */
  }
  function has(id: string) {
    return items.value.some((item) => item.project_id === id)
  }
  function toggle(project: ISearchHit) {
    const next = has(project.project_id)
      ? items.value.filter((item) => item.project_id !== project.project_id)
      : [...items.value, project]
    try {
      localStorage.setItem(key, JSON.stringify(next))
      items.value = next
    } catch {
      sideTip.show(pclText('Proto.Storage.FavoritesFailed'), 'warn')
    }
  }
  return { items, has, toggle }
})
