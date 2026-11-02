<script setup lang="ts">
import PListItem from '@/components/widget/PListItem.vue'
import PCard from '@/components/widget/PCard.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PSearchBox from '@/components/widget/PSearchBox.vue'
import PHint from '@/components/widget/PHint.vue'
import PButton from '@/components/widget/PButton.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import { useModal } from '@/composables/useModal'
import sideTip from '@/composables/sideTip'
import { useSelectedInstance } from '@/stores/gameLaunch'
import { useRepositoriesStore } from '@/stores/repositories'
import type GameInstance from '@/types/gameInstance'
import { pluginShowText, instanceIcon, instanceLoader, type pluginType } from '@/types/gameInfo'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { pclText } from '@/api/pclLocale'
import { useInstanceFavorites } from '@/composables/useInstanceFavorites'

const route = useRoute()
const router = useRouter()
const repos = useRepositoriesStore()
const selected = useSelectedInstance()
const text = pclText
const instances = ref<GameInstance[]>([])
const loading = ref(false)
const error = ref('')
const selecting = ref(false)
const query = ref('')
const { favorites, toggleFavorite: toggleFavoritePath } = useInstanceFavorites()
function toggleFavorite(item: GameInstance) {
  const error = toggleFavoritePath(item.directory)
  if (error) sideTip.show(error, 'warn')
}
const filtered = computed(() =>
  instances.value.filter((item) =>
    `${item.name} ${item.version} ${Object.entries(item.pluginsVersion || {})
      .map(([key, version]) => `${pluginShowText[key as pluginType] || key} ${key} ${version}`)
      .join(' ')}`
      .toLowerCase()
      .includes(query.value.trim().toLowerCase()),
  ),
)
function loader(item: GameInstance) {
  return instanceLoader(item.pluginsVersion || {})
}
function tags(item: GameInstance) {
  return [
    item.version,
    ...Object.entries(item.pluginsVersion || {})
      .filter(([key]) => key !== 'vanilla')
      .map(([key, version]) => `${pluginShowText[key as pluginType] || key} ${version}`),
  ]
}
const groups = computed(() => {
  const modded = filtered.value.filter((item) => loader(item))
  const regular = filtered.value.filter((item) => !loader(item))
  const kinds = [...new Set(instances.value.map(loader).filter(Boolean))]
  const labels: Partial<Record<pluginType, string>> = {
    forge: 'Forge',
    neoforge: 'NeoForge',
    cleanroom: 'Cleanroom',
    'laby-mod': 'LabyMod',
    quilt: 'Quilt',
    fabric: 'Fabric',
    legacyfabric: 'Fabric',
    liteloader: 'LiteLoader',
  }
  const label = kinds.length === 1 ? labels[kinds[0]!] : undefined
  return [
    {
      title: text('Select.Instance.Card.Favorites'),
      items: filtered.value.filter((item) => favorites.value.includes(item.directory)),
      favorite: true,
    },
    { title: text(`Select.Instance.Card.${label || 'Modable'}`), items: modded, favorite: false },
    { title: text('Select.Instance.Card.Regular'), items: regular, favorite: false },
  ].filter((group) => group.items.length)
})
let request = 0
async function load() {
  const current = ++request
  loading.value = true
  error.value = ''
  instances.value = []
  try {
    if (!isTauri()) return
    if (!repos.repositories) await repos.fetchFromBackend()
    const index = Number(route.params.repository || 0)
    if (!Number.isInteger(index) || index < 0 || !repos.repositories?.[index]) return
    await repos.selectRepository(index)
    if (current !== request) return
    const items = await repos.getInstancesInRepository(index)
    if (current === request) instances.value = items
  } catch (err) {
    if (current === request) error.value = pclText('Proto.Error.Instances', [String(err)])
  } finally {
    if (current === request) loading.value = false
  }
}
async function select(instance: GameInstance) {
  if (selecting.value) return
  selecting.value = true
  try {
    const saved = await invoke<GameInstance>('select_instance', {
      repository_index: Number(route.params.repository || 0),
      instance_name: instance.name,
    })
    selected.setInstance(saved)
    sideTip.show(pclText('Proto.Instance.Selected', [instance.name]), 'success')
    await router.push('/home')
  } catch (err) {
    sideTip.show(pclText('Proto.Error.SelectInstance', [String(err)]), 'warn')
  } finally {
    selecting.value = false
  }
}
async function settings(item: GameInstance) {
  if (selecting.value) return
  selecting.value = true
  try {
    const saved = await invoke<GameInstance>('select_instance', {
      repository_index: Number(route.params.repository || 0),
      instance_name: item.name,
    })
    selected.setInstance(saved)
    await router.push('/instance_setting/overview')
  } catch (err) {
    sideTip.show(pclText('Proto.Error.SelectInstance', [String(err)]), 'warn')
  } finally {
    selecting.value = false
  }
}
async function folderAction(item: GameInstance, remove: boolean) {
  const repositoryIndex = Number(route.params.repository || 0)
  if (remove) {
    const modal = useModal()
    const result = await modal.open({
      title: text('Select.Instance.Delete.ConfirmTitle'),
      content: `${text('Select.Instance.Delete.ConfirmMessage', [item.name])}\n${text('Select.Instance.Delete.IsolatedWarning')}`,
      buttons: [
        { content: text('Common.Action.Delete'), type: 'warn', operation: () => modal.close(true) },
        { content: text('Common.Action.Cancel'), operation: () => modal.close(false) },
      ],
    })
    if (!result.confirmed) return
  }
  try {
    await invoke('instance_folder_action', {
      repository_index: repositoryIndex,
      instance_name: item.name,
      instance_directory: item.directory,
      delete: remove,
    })
    if (remove) {
      if (selected.instance_info?.directory === item.directory) selected.setInstance(null)
      if (favorites.value.includes(item.directory)) toggleFavorite(item)
      sideTip.show(text('Select.Instance.Delete.RecycleBinSuccess', [item.name]), 'success')
      await load()
    }
  } catch (err) {
    sideTip.show(String(err), 'warn')
  }
}
watch(
  () => route.params.repository,
  () => {
    query.value = ''
    void load()
  },
  { immediate: true },
)
</script>
<template>
  <section class="pcl-page select-page">
    <PSearchBox
      v-if="instances.length || query"
      v-model="query"
      :placeholder="text('Select.Instance.Search.Hint')"
    />
    <PLoading v-if="loading" :loading-text="text('Select.Instance.Loading')" />
    <PHint v-else-if="error" severity="error"
      >{{ error
      }}<PButton inline @click="load">{{
        pclText('Launch.Homepage.LoadFailed.Retry')
      }}</PButton></PHint
    >
    <PCard v-else-if="!instances.length" hide-title class="empty-card"
      ><div class="empty-content">
        <h2>{{ text('Select.Instance.Empty.Title') }}</h2>
        <hr />
        <p>{{ text('Select.Instance.Empty.Message') }}</p>
        <PButton type="tint" @click="() => router.push('/download/game')">{{
          text('Select.Instance.Empty.Download')
        }}</PButton>
      </div></PCard
    >
    <PCard v-else-if="!filtered.length" hide-title class="empty-card"
      ><div class="empty-content">
        <h2>{{ text('Select.Instance.EmptySearch.Title') }}</h2>
        <hr />
        <p>{{ text('Select.Instance.Search.NoResult', [query]) }}</p>
      </div></PCard
    >
    <template v-else
      ><PCard
        v-for="group in groups"
        :key="group.title"
        :title="group.favorite ? group.title : `${group.title} (${group.items.length})`"
        ><div class="instance-list">
          <PListItem
            v-for="item in group.items"
            :key="item.directory"
            :title="item.name"
            :tags="tags(item)"
            :icon="instanceIcon(item.version, item.pluginsVersion || {})"
            clickable
            @click="() => select(item)"
            @contextmenu.prevent="settings(item)"
          >
            <template #icon-actions>
              <button
                class="pcl-icon-button"
                type="button"
                :title="
                  text(
                    favorites.includes(item.directory)
                      ? 'Select.Instance.Unfavorite'
                      : 'Select.Instance.Favorite',
                  )
                "
                :aria-label="
                  text(
                    favorites.includes(item.directory)
                      ? 'Select.Instance.Unfavorite'
                      : 'Select.Instance.Favorite',
                  )
                "
                :aria-pressed="favorites.includes(item.directory)"
                @click="toggleFavorite(item)"
              >
                <PSvgIcon
                  :name="
                    favorites.includes(item.directory) ? 'lucide/heart-filled' : 'lucide/heart'
                  "
                  :size="15"
                />
              </button>
              <button
                class="pcl-icon-button"
                type="button"
                :title="text('Select.Instance.OpenFolder')"
                :aria-label="text('Select.Instance.OpenFolder')"
                @click="folderAction(item, false)"
              >
                <PSvgIcon name="lucide/folder-open" :size="15" />
              </button>
              <button
                class="pcl-icon-button"
                type="button"
                :title="text('Common.Action.Delete')"
                :aria-label="text('Common.Action.Delete')"
                @click="folderAction(item, true)"
              >
                <PSvgIcon name="lucide/trash-2" :size="15" />
              </button>
              <button
                class="pcl-icon-button"
                type="button"
                :title="text('Select.Instance.Settings')"
                :aria-label="text('Select.Instance.Settings')"
                @click="settings(item)"
              >
                <PSvgIcon name="lucide/settings" :size="15" />
              </button>
            </template>
          </PListItem></div></PCard
    ></template>
  </section>
</template>
<style scoped>
.select-page {
  padding: 15px 25px 25px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.empty-card {
  margin: 25px 15px;
}
.empty-content {
  padding: 17px 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.empty-content h2 {
  font-size: 18px;
  font-weight: normal;
}
.empty-content p {
  padding: 0 10px;
  white-space: pre-line;
}
.empty-content hr {
  border: 0;
  height: 2px;
  background: var(--color-tint-light);
}
.instance-list {
  padding: 0 20px 15px;
}
</style>
