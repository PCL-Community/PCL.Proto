<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'
import { showUnavailable } from '@/util/showUnavailable'
import { pclText } from '@/api/pclLocale'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import { computed, ref, watch } from 'vue'
import PLocalModItem from '@/components/widget/PLocalModItem.vue'
import PSearchBox from '@/components/widget/PSearchBox.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PHint from '@/components/widget/PHint.vue'
import PIconTextButton from '@/components/widget/PIconTextButton.vue'
import PSelectionToolbar from '@/components/widget/PSelectionToolbar.vue'
import { popupNativeMenu } from '@/util/nativeMenu'
import { useModal } from '@/composables/useModal'
import { resourceUrl, useInstanceResources } from '@/composables/useInstanceResources'
import type { InstanceResourceKind } from '@/api/instanceResources'

const route = useRoute()
const router = useRouter()
const kind = computed<InstanceResourceKind>(() => {
  switch (route.meta.resourceType) {
    case 'ResourcePack':
      return 'resourcepack'
    case 'Shader':
      return 'shader'
    case 'Schematic':
      return 'schematic'
    default:
      return 'mod'
  }
})
const resources = useInstanceResources({ kind: () => kind.value })
// Schematic users are shown the list; unsupported mods surface through the filter instead.
const schematicAvailable = true
const query = ref('')
const filter = ref('All')
const selected = ref(new Set<string>())
const sort = ref('ResourceName')
watch(kind, () => {
  selected.value.clear()
  filter.value = 'All'
  query.value = ''
})
watch(resources.items, (entries, previous) => {
  const paths = new Set(entries.map((entry) => entry.path))
  for (const path of selected.value) {
    if (paths.has(path)) continue
    const old = previous.find((entry) => entry.path === path)
    const renamed =
      old && entries.find((entry) => entry.name === old.name && entry.enabled !== old.enabled)
    selected.value.delete(path)
    if (renamed) selected.value.add(renamed.path)
  }
})

function formatSize(size: number) {
  if (!size) return ''
  if (size < 1024 * 1024) return `${Math.max(1, Math.round(size / 1024))} KB`
  return `${(size / 1024 / 1024).toFixed(1)} MB`
}

function formatDate(seconds: number) {
  if (!seconds) return ''
  return new Date(seconds * 1000).toLocaleDateString()
}

const items = computed(() =>
  resources.items.value.map((entry) => {
    const fileName = entry.name.replace(/\.(jar|zip|litemod)$/i, '')
    const detail =
      entry.displayName || entry.description
        ? [fileName, entry.description?.replace(/\s+/g, ' ')].filter(Boolean).join(': ')
        : [formatSize(entry.size), formatDate(entry.modified)].filter(Boolean).join(' · ')
    return {
      id: entry.path,
      title: entry.displayName || fileName,
      description: detail,
      enabled: entry.enabled,
      icon: entry.iconData || resourceUrl(entry.token),
      version: entry.version,
      fileName: entry.name,
      size: entry.size,
      created: entry.created,
    }
  }),
)

// 筛选计数只统计搜索命中的条目，口径与 CE 一致
const searchedItems = computed(() =>
  items.value.filter((item) =>
    `${item.title} ${item.description || ''}`.toLowerCase().includes(query.value.toLowerCase()),
  ),
)
const filterOptions = computed(() => {
  const total = searchedItems.value.length
  const enabled = searchedItems.value.filter((item) => item.enabled !== false).length
  const disabled = total - enabled
  return [
    { key: 'All', count: total, visible: true },
    { key: 'Enabled', count: enabled, visible: enabled > 0 && enabled < total },
    { key: 'Disabled', count: disabled, visible: disabled > 0 },
  ]
})
function filterLabel(option: { key: string; count: number }) {
  if (option.key === 'All' && query.value) return text('Instance.Resource.Filter.SearchResult')
  return text(`Instance.Resource.Filter.${option.key}WithCount`, [option.count])
}

const visibleItems = computed(() => {
  const matched = searchedItems.value.filter(
    (item) =>
      filter.value === 'All' ||
      (filter.value === 'Enabled' && item.enabled !== false) ||
      (filter.value === 'Disabled' && item.enabled === false),
  )
  return [...matched].sort((a, b) => {
    if (sort.value === 'FileSize') return b.size - a.size || a.title.localeCompare(b.title)
    if (sort.value === 'AddTime') return b.created - a.created || a.title.localeCompare(b.title)
    return (sort.value === 'FileName' ? a.fileName : a.title).localeCompare(
      sort.value === 'FileName' ? b.fileName : b.title,
    )
  })
})

function chooseSort(event: MouseEvent) {
  void popupNativeMenu(
    event.currentTarget as Element,
    ['ResourceName', 'FileName', 'AddTime', 'FileSize'].map((key) => ({
      text: text(`Instance.Resource.Sort.${key}`),
      action: () => {
        sort.value = key
      },
    })),
  )
}

function select(id: string, value: boolean) {
  if (value) selected.value.add(id)
  else selected.value.delete(id)
}

function toggleItem(id: string) {
  const entry = resources.items.value.find((item) => item.path === id)
  if (entry) void resources.toggle(entry)
}
function removeItem(id: string) {
  const entry = resources.items.value.find((item) => item.path === id)
  if (entry) {
    selected.value.delete(id)
    void resources.remove([entry])
  }
}
function showInfo(id: string) {
  const entry = resources.items.value.find((item) => item.path === id)
  if (!entry) return
  const modal = useModal()
  void modal.open({
    title: entry.displayName || entry.name,
    plainText: true,
    content: [entry.version, entry.description, entry.name, formatSize(entry.size)]
      .filter(Boolean)
      .join('\n'),
    buttons: [{ content: text('Common.Action.Close'), operation: () => modal.close() }],
  })
}

function runSelection(action: string) {
  const chosen = resources.items.value.filter((entry) => selected.value.has(entry.path))
  if (action === 'Enable') void resources.setEnabled(chosen, true)
  else if (action === 'Disable') void resources.setEnabled(chosen, false)
  else if (action === 'Delete') void resources.remove(chosen)
  else {
    showUnavailable()
    return
  }
  selected.value.clear()
}

const selectionActions = [
  { id: 'Update', key: 'Instance.Resource.Update', icon: 'upload' },
  { id: 'Enable', key: 'Instance.Resource.Enable', icon: 'circle-check' },
  { id: 'Disable', key: 'Instance.Resource.Disable', icon: 'circle-minus' },
  { id: 'Favorites', key: 'Instance.Resource.Favorite', icon: 'heart' },
  { id: 'Share', key: 'Instance.Resource.ShareSelected', icon: 'share-2' },
  { id: 'Delete', key: 'Common.Action.Delete', icon: 'trash-2' },
]
const selectedEntries = computed(() =>
  resources.items.value.filter((entry) => selected.value.has(entry.path)),
)
function selectionDisabled(action: string) {
  if (action === 'Enable') return !selectedEntries.value.some((entry) => !entry.enabled)
  if (action === 'Disable') return !selectedEntries.value.some((entry) => entry.enabled)
  return ['Update', 'Favorites', 'Share'].includes(action)
}
function downloadResources() {
  const category = String(route.meta.downloadType || 'mod')
  return router.push({
    path: `/download/${category === 'schematic' ? 'mod' : category}`,
    query: { instance: 'selected' },
  })
}

const loading = computed(() => resources.loading.value)
const error = computed(() => resources.error.value)
const text = pclText
</script>
<template>
  <div class="pcl-page resource-page">
    <PLoading v-if="loading" :loading-text="text('Instance.Resource.Loading')" />
    <div v-else>
      <PCard
        v-if="route.meta.resourceType === 'Schematic' && !schematicAvailable"
        hide-title
        class="pcl-unavailable"
        ><h2>{{ text('Instance.Resource.Schematic.Unavailable.Title') }}</h2>
        <div class="pcl-divider" />
        <p>{{ text('Instance.Resource.Schematic.Unavailable.Message') }}</p>
        <div class="resource-actions">
          <PButton type="tint" @click="downloadResources">{{
            text('Instance.Resource.Schematic.DownloadMod')
          }}</PButton
          ><PButton @click="() => router.push('/instance_select')">{{
            text('Instance.Resource.Mod.Disabled.SelectInstance')
          }}</PButton>
        </div></PCard
      >
      <div v-else style="display: grid">
        <div
          data-control="PanAllBack"
          style="grid-row: 1 / span 1; grid-column: 1 / span 1; display: grid"
        >
          <div data-control="PanBack" style="grid-row: 1 / span 1; grid-column: 1 / span 1">
            <div v-if="items.length && !error" class="resource-main">
              <PSearchBox v-model="query" :placeholder="text('Instance.Resource.Search.Hint')" />
              <PCard hide-title class="resource-manage"
                ><div class="resource-actions">
                  <PButton
                    v-for="action in [
                      {
                        id: 'BtnManageOpen',
                        key: 'Common.Action.OpenFolder',
                        run: () => resources.openFolder(),
                      },
                      {
                        id: 'BtnManageInstall',
                        key: 'Instance.Resource.InstallFromFiles',
                        run: () => resources.importFiles(),
                      },
                      {
                        id: 'BtnManageDownload',
                        key: 'Instance.Resource.DownloadNew',
                        run: downloadResources,
                      },
                      {
                        id: 'BtnManageSelectAll',
                        key: 'Instance.Resource.SelectAll',
                        run: () => visibleItems.forEach((item) => selected.add(item.id)),
                      },
                      {
                        id: 'BtnManageInfoExport',
                        key: 'Instance.Resource.ExportInfo',
                        run: showUnavailable,
                      },
                    ]"
                    :key="action.id"
                    :type="action.id === 'BtnManageOpen' ? 'tint' : 'default'"
                    @click="action.run()"
                    >{{ text(action.key) }}</PButton
                  ><PButton
                    v-if="kind === 'mod'"
                    :title="text('Instance.Resource.Mod.JarInJar.Entry.ToolTip')"
                    @click="router.push('/instance_setting/mod/jarinjar')"
                    >{{ text('Instance.Resource.Mod.JarInJar.Entry') }}</PButton
                  >
                </div></PCard
              >
              <PCard hide-title
                ><div class="resource-actions">
                  <div class="resource-filter">
                    <button
                      v-for="option in filterOptions"
                      v-show="option.visible || filter === option.key"
                      :key="option.key"
                      type="button"
                      class="pcl-chip"
                      :class="{ active: filter === option.key }"
                      @click="filter = option.key"
                    >
                      {{ filterLabel(option) }}
                    </button>
                  </div>
                  <PIconTextButton icon="lucide/arrow-up-down" @click="chooseSort">{{
                    text('Instance.Resource.Sort.Text', [text(`Instance.Resource.Sort.${sort}`)])
                  }}</PIconTextButton>
                </div>
                <div class="resource-list">
                  <PLocalModItem
                    v-for="item in visibleItems"
                    :key="item.id"
                    v-bind="item"
                    :togglable="kind === 'mod'"
                    :selected="selected.has(item.id)"
                    @select="(value) => select(item.id, value)"
                    @toggle="toggleItem(item.id)"
                    @update="showUnavailable()"
                    @info="showInfo(item.id)"
                    @open="resources.openFolder()"
                    @remove="removeItem(item.id)"
                  />
                  <p v-if="!visibleItems.length" class="pcl-empty">
                    {{ pclText('Download.Comp.List.NoMatchingResults') }}
                  </p>
                </div></PCard
              >
            </div>
          </div>
          <PHint v-if="error" severity="error" class="resource-status"
            >{{ error
            }}<PButton inline @click="resources.refresh()">{{
              text('Common.Action.Refresh')
            }}</PButton></PHint
          >
          <PCard
            v-if="!error && !items.length"
            data-control="PanEmpty"
            style="
              margin: 40px;
              grid-row: 1 / span 1;
              grid-column: 1 / span 1;
              justify-self: center;
              align-self: center;
            "
            hide-title
          >
            <div style="margin: 17px 20px; display: flex; flex-direction: column">
              <span
                data-control="TxtEmptyTitle"
                style="margin: 0px 0px 9px 0px; font-size: 19px; color: var(--color-tint)"
                class="pcl-text"
                >{{ text('Instance.Resource.Empty.Title') }}</span
              >
              <div style="height: 2px" class="pcl-divider" />
              <span
                data-control="TxtEmptyDescription"
                style="margin: 10px 10px 0px 10px"
                class="pcl-text"
                >{{ text('Instance.Resource.Empty.Description') }}</span
              >
              <div
                style="
                  margin: 20px 10px 5px 10px;
                  display: flex;
                  flex-direction: row;
                  align-items: center;
                  flex-wrap: wrap;
                "
              >
                <PButton
                  data-control="BtnHintInstall"
                  style="height: 35px; min-width: 130px; margin: 0px 5px; padding: 0px 12px"
                  @click="resources.importFiles()"
                  type="tint"
                  >{{ text('Instance.Resource.InstallFromFiles') }}</PButton
                >
                <PButton
                  data-control="BtnHintDownload"
                  style="height: 35px; min-width: 130px; margin: 0px 5px; padding: 0px 12px"
                  @click="downloadResources"
                  >{{ text('Instance.Resource.DownloadNew') }}</PButton
                >
                <PButton
                  data-control="BtnHintOpen"
                  style="height: 35px; min-width: 130px; margin: 0px 5px; padding: 0px 12px"
                  @click="resources.openFolder()"
                  >{{ text('Common.Action.OpenFolder') }}</PButton
                >
              </div>
            </div>
          </PCard>
        </div>
      </div>
    </div>
    <PSelectionToolbar :count="selected.size">
      <PIconTextButton
        v-for="action in selectionActions.filter(
          (action) =>
            (kind === 'mod' || !['Enable', 'Disable'].includes(action.id)) &&
            (kind !== 'schematic' || !['Update', 'Favorites', 'Share'].includes(action.id)),
        )"
        :key="action.id"
        :icon="`lucide/${action.icon}`"
        :disabled="selectionDisabled(action.id)"
        @click="runSelection(action.id)"
        >{{ text(action.key) }}</PIconTextButton
      ><PIconTextButton icon="lucide/x" @click="() => selected.clear()">{{
        text('Instance.Resource.CancelSelection')
      }}</PIconTextButton>
    </PSelectionToolbar>
  </div>
</template>

<style scoped>
.resource-page {
  display: flex;
  flex-direction: column;
}

.resource-main {
  padding: 25px;
  display: flex;
  flex-direction: column;
  gap: 15px;
}
.resource-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 15px;
  padding: 15px;
}
.resource-manage .resource-actions :deep(button) {
  min-width: 110px;
  min-height: 34px;
}
.resource-filter {
  display: flex;
  align-items: center;
  margin-right: auto;
}
.resource-list {
  padding: 0 20px 20px;
}
.resource-status {
  margin: 40px;
  grid-area: 1 / 1;
}
</style>
