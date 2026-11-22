<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, shallowRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke, isTauri } from '@tauri-apps/api/core'
import PCard from '@/components/widget/PCard.vue'
import PVirtualList from '@/components/widget/PVirtualList.vue'
import PListItem from '@/components/widget/PListItem.vue'
import PVersionActions from '@/components/widget/PVersionActions.vue'
import PButton from '@/components/widget/PButton.vue'
import PTextBox from '@/components/widget/PTextBox.vue'
import PSearchBox from '@/components/widget/PSearchBox.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PHint from '@/components/widget/PHint.vue'
import PLoading from '@/components/widget/PLoading.vue'
import { getMinecraftVersions, type IVersionShow } from '@/api/gameVersions'
import { pclFormatLocale, pclText } from '@/api/pclLocale'
import { isTaskCancelled, useTaskManager, type LoaderInstall } from '@/stores/task'
import { prepareLoaderJava } from '@/api/loaderJava'
import sideTip from '@/composables/sideTip'
import { useSelectedInstance } from '@/stores/gameLaunch'
import { showIconPath, loaderIcon, instanceIcon, type pluginType } from '@/types/gameInfo'
import type { McPluginReport } from '@/types/mcPlugin'
import {
  instanceDefaultName,
  installWarnings,
  loaderConflict,
  loaderVisible,
  selectLoader,
} from '@/api/installSelection'
import { saveMinecraftVersion } from '@/api/versionActions'
import { getCompVersions } from '@/api/compProjects'
import { FloatButtonType, useFloatButton } from '@/composables/useFloatButton'
import type { ISetupOption } from '@/types/setupOption'

const props = defineProps<{ manual?: boolean; modify?: boolean }>()
const route = useRoute()
const router = useRouter()
const versions = shallowRef<IVersionShow>()
const loading = ref(false)
const error = ref<string | null>(null)
const installing = ref(false)
const { floatButtonState, setFloatButton } = useFloatButton()
const instance = useSelectedInstance()
const versionId = computed(() =>
  props.modify ? instance.instance_info?.version || '' : String(route.query.version_id || ''),
)
const instanceName = ref(versionId.value)
const instanceNameEdited = ref(false)
const plugins = reactive<
  Record<string, { loading: boolean; versions: McPluginReport[]; error: string }>
>({})
const selected = reactive<Record<string, string>>({})
type FabricApiFile = {
  url: string
  size: number
  sha1: string
  filename: string
}
const fabricApiFiles: Record<string, FabricApiFile> = {}
let fabricApiAutoSelected = false
const activatedLoaders = new Set<string>()
const selectedPlugins = computed(
  () =>
    Object.fromEntries(
      Object.entries(selected).map(([key, value]) => [
        key === 'LabyMod' ? 'laby-mod' : key.toLowerCase(),
        value,
      ]),
    ) as Partial<Record<pluginType, string>>,
)
const activeLoader = ref('')
const activeLoaderVersion = computed<string>({
  get: () => (activeLoader.value ? selected[activeLoader.value] || '' : ''),
  set: (version) => {
    if (activeLoader.value && version) chooseLoader(activeLoader.value, version)
  },
})
const activeLoaderOptions = computed<ISetupOption[]>(() =>
  (plugins[activeLoader.value]?.versions || []).map((version) => ({
    key: version.version,
    text: version.version,
  })),
)
function activateLoader(loader: string) {
  if (activeLoader.value === loader && !selected[loader]) {
    activeLoader.value = ''
    activatedLoaders.delete(loader)
    return
  }
  if (selected[loader]) {
    selectLoader(selected, loader, '')
    delete selected[loader]
    activatedLoaders.delete(loader)
    if (loader === 'Fabric' || loader === 'FabricApi') fabricApiAutoSelected = false
    if (!instanceNameEdited.value)
      instanceName.value = instanceDefaultName(versionId.value, selected)
    if (activeLoader.value === loader) activeLoader.value = ''
    return
  }
  activeLoader.value = loader
  activatedLoaders.add(loader)
  if (loader === 'FabricApi' && !selected.Fabric) {
    const fabricLatest = plugins.Fabric?.versions[0]?.version
    if (fabricLatest) chooseLoader('Fabric', fabricLatest)
  }
  const latest = plugins[loader]?.versions[0]?.version
  if (latest && !selected[loader]) chooseLoader(loader, latest)
  activeLoader.value = loader
}
function chooseLoader(loader: string, version: string) {
  selectLoader(selected, loader, version)
  if (loader === 'FabricApi') fabricApiAutoSelected = true
  if (!selected.Fabric) fabricApiAutoSelected = false
  if (loader === 'Fabric' && version && !fabricApiAutoSelected && plugins.FabricApi?.versions[0]) {
    fabricApiAutoSelected = true
    chooseLoader('FabricApi', plugins.FabricApi.versions[0].version)
  }
  activeLoader.value = loader
  if (!instanceNameEdited.value) instanceName.value = instanceDefaultName(versionId.value, selected)
}
const warnings = computed(() => installWarnings(selected, versionId.value))
function loaderSummary(loader: string) {
  return (
    selected[loader] ||
    plugins[loader]?.versions[0]?.version ||
    (plugins[loader]?.loading
      ? pclText('Proto.Download.Loading')
      : pclText('Proto.Download.SelectVersion'))
  )
}
const loaders = [
  'Forge',
  'Cleanroom',
  'NeoForge',
  'Fabric',
  'LegacyFabric',
  'FabricApi',
  'LegacyFabricApi',
  'LabyMod',
  'OptiFine',
  'OptiFabric',
  'LiteLoader',
]
const versionedLoaders = ['Forge', 'NeoForge', 'Fabric', 'LegacyFabric', 'FabricApi']
const visibleLoaders = computed(() =>
  loaders.filter((loader) => loaderVisible(loader, versionId.value, selected)),
)
const activeLoaderMessage = computed(() => {
  const loader = activeLoader.value
  const plugin = loader ? plugins[loader] : undefined
  if (!loader || plugin?.loading || plugin?.error || plugin?.versions.length) return ''
  return versionedLoaders.includes(loader)
    ? pclText('Download.Install.State.NoAvailableVersion')
    : pclText('Proto.NotImplemented.Loader')
})
watch(
  visibleLoaders,
  (list) => {
    if (activeLoader.value && !list.includes(activeLoader.value)) activeLoader.value = ''
  },
  { immediate: true },
)
const categories = [
  { key: 'release', label: 'Release', icon: showIconPath.grass },
  { key: 'snapshot', label: 'Development', icon: showIconPath.command },
  { key: 'old', label: 'BeforeRelease', icon: showIconPath.stone },
  { key: 'fool', label: 'AprilFools', icon: showIconPath.gold },
] as const
type VersionCategory = (typeof categories)[number]['key']
const selectedCategory = ref<VersionCategory>('release')
const selectedCategoryIndex = computed(() =>
  Math.max(
    0,
    categories.findIndex((category) => category.key === selectedCategory.value),
  ),
)
const versionQuery = ref('')
const visibleVersions = computed(() => {
  const list = versions.value?.[selectedCategory.value] || []
  const query = versionQuery.value.trim().toLowerCase()
  return query ? list.filter((version) => version.id.toLowerCase().includes(query)) : list
})
const selectedCategoryInfo = computed(
  () => categories.find((category) => category.key === selectedCategory.value) || categories[0],
)
async function load() {
  loading.value = true
  error.value = null
  try {
    const data = await getMinecraftVersions()
    versions.value = data
    if (!props.manual && !props.modify && !route.query.version_id) {
      const initial = data.latest.release || data.latest.snapshot
      if (initial) {
        void router.replace({
          path: '/download/game',
          query: {
            version_id: initial.id,
            version_type: data.latest.release ? 'release' : 'snapshot',
          },
        })
      }
    }
  } catch (err) {
    error.value = pclText('Proto.Error.Versions', [String(err)])
  } finally {
    loading.value = false
  }
}
function choose(id: string, type: string) {
  if (props.manual) {
    void saveMinecraftVersion(id)
    return
  }
  void router.replace({
    path: '/download/game',
    query: { version_id: id, version_type: type },
  })
}
let request = 0
async function loadFabricApi(id: string, current: number) {
  plugins.FabricApi = { loading: true, versions: [], error: '' }
  try {
    if (!isTauri()) throw new Error(pclText('Proto.Desktop.Loaders'))
    const target = id
      .replace('∞', 'infinite')
      .replace('Combat Test 7c', '1.16_combat-3')
      .toLowerCase()
    const list = (await getCompVersions('fabric-api'))
      .filter(
        (version) =>
          version.loaders.includes('fabric') &&
          version.game_versions.some((game) => game.toLowerCase() === target),
      )
      .sort((a, b) => Date.parse(b.date_published) - Date.parse(a.date_published))
    const versions: McPluginReport[] = []
    const files: Record<string, FabricApiFile> = {}
    for (const version of list) {
      const file = version.files.find((file) => file.primary) || version.files[0]
      const sha1 = file?.hashes?.sha1
      if (!file || !sha1 || !/^https:\/\//i.test(file.url)) continue
      files[version.version_number] = {
        url: file.url,
        size: file.size,
        sha1,
        filename: file.filename,
      }
      versions.push({
        version: version.version_number,
        stable: version.version_type === 'release',
        release_time: version.date_published,
      })
    }
    if (current !== request) return
    Object.assign(fabricApiFiles, files)
    plugins.FabricApi = { loading: false, versions, error: '' }
    if (selected.Fabric && !fabricApiAutoSelected && versions[0]) {
      fabricApiAutoSelected = true
      chooseLoader('FabricApi', versions[0].version)
    }
    if (activatedLoaders.has('FabricApi') && !selected.FabricApi && versions[0])
      chooseLoader('FabricApi', versions[0].version)
  } catch (err) {
    if (current === request)
      plugins.FabricApi = {
        loading: false,
        versions: [],
        error: pclText('Proto.Error.Versions', [String(err)]),
      }
  }
}
async function loadLoader(name: string, id: string, current: number) {
  if (name === 'FabricApi') return loadFabricApi(id, current)
  plugins[name] = { loading: true, versions: [], error: '' }
  try {
    if (!isTauri()) throw new Error(pclText('Proto.Desktop.Loaders'))
    const list = await invoke<McPluginReport[]>('get_plugin_versions', {
      plugin_type: name.toLowerCase(),
      mc_version: id,
    })
    if (current === request) {
      plugins[name] = { loading: false, versions: list, error: '' }
      if (activatedLoaders.has(name) && !selected[name] && list[0])
        chooseLoader(name, list[0].version)
    }
  } catch (err) {
    if (current === request)
      plugins[name] = {
        loading: false,
        versions: [],
        error: pclText('Proto.Error.Versions', [String(err)]),
      }
  }
}
watch(
  versionId,
  async (id) => {
    instanceName.value = id
    instanceNameEdited.value = false
    activeLoader.value = ''
    Object.keys(selected).forEach((key) => delete selected[key])
    Object.keys(fabricApiFiles).forEach((key) => delete fabricApiFiles[key])
    activatedLoaders.clear()
    fabricApiAutoSelected = false
    const current = ++request
    if (!id) return
    await Promise.all(
      ['Forge', 'NeoForge', 'Fabric', 'LegacyFabric', 'FabricApi'].map((name) =>
        loadLoader(name, id, current),
      ),
    )
  },
  { immediate: true },
)
async function install() {
  if (installing.value) return
  if (props.modify) {
    sideTip.show(pclText('Proto.NotImplemented.ModifyCore'))
    return
  }
  if (
    !instanceName.value.trim() ||
    /[\\/:*?"<>|\u0000-\u001f]/.test(instanceName.value) ||
    ['.', '..'].includes(instanceName.value.trim())
  ) {
    sideTip.show(pclText('Proto.Instance.InvalidName'), 'warn')
    return
  }
  if (!isTauri()) {
    sideTip.show(pclText('Proto.Desktop.Download'), 'warn')
    return
  }
  if (
    Object.entries(selected).some(
      ([name, value]) =>
        !['Forge', 'NeoForge', 'Fabric', 'LegacyFabric', 'FabricApi'].includes(name) && !!value,
    )
  ) {
    sideTip.show(pclText('Proto.NotImplemented.LoaderInstall'), 'warn')
    return
  }
  if (selected.FabricApi && !fabricApiFiles[selected.FabricApi]) {
    sideTip.show(pclText('Download.Comp.QuickDownload.Hint.NoFile'), 'warn')
    return
  }
  installing.value = true
  try {
    const game = versionId.value
    const name = instanceName.value.trim()
    const loader: LoaderInstall | undefined = selected.Fabric
      ? {
          family: 'fabric',
          version: selected.Fabric,
          api: fabricApiFiles[selected.FabricApi] || null,
        }
      : selected.LegacyFabric
        ? { family: 'legacyfabric', version: selected.LegacyFabric }
        : selected.Forge
          ? { family: 'forge', version: selected.Forge }
          : selected.NeoForge
            ? { family: 'neoforge', version: selected.NeoForge }
            : undefined
    if (!(await prepareLoaderJava(game, loader))) return
    void useTaskManager()
      .startDownloadMCVersion(game, name, loader)
      .catch((err) => {
        if (!isTaskCancelled(err))
          sideTip.show(pclText('Proto.Error.Download', [String(err)]), 'warn')
      })
    await router.push('/download/game')
  } catch (err) {
    sideTip.show(pclText('Proto.Error.Download', [String(err)]), 'warn')
  } finally {
    installing.value = false
  }
}
onMounted(() => {
  if (!versionId.value) void load()
  watch(
    versionId,
    (id) => {
      if (id && !props.manual) {
        floatButtonState.action = install
        setFloatButton(props.modify ? FloatButtonType.ModifyInstance : FloatButtonType.DownloadGame)
        floatButtonState.visible = true
      } else if (floatButtonState.action === install) {
        floatButtonState.action = undefined
        const hasTasks = useTaskManager().activeTaskCount > 0
        if (hasTasks) setFloatButton(FloatButtonType.TaskManage)
        floatButtonState.visible = hasTasks
      }
    },
    { immediate: true },
  )
})
onUnmounted(() => {
  if (floatButtonState.action === install) {
    floatButtonState.action = undefined
    const hasTasks = useTaskManager().activeTaskCount > 0
    if (hasTasks) setFloatButton(FloatButtonType.TaskManage)
    floatButtonState.visible = hasTasks
  }
})
watch(versionId, (id) => {
  if (!id && !versions.value) void load()
})
</script>
<template>
  <section class="pcl-page download-game-page">
    <p v-if="modify && !versionId" class="pcl-empty">
      {{ pclText('Proto.Instance.SelectFirst') }}
    </p>
    <PLoading v-if="loading" :loading-text="pclText('Download.Version.LoadingList')" />
    <PHint v-else-if="error" severity="error"
      >{{ error }}
      <PButton inline @click="load">{{
        pclText('Launch.Homepage.LoadFailed.Retry')
      }}</PButton></PHint
    >
    <div
      v-else-if="versions"
      class="download-game-columns"
      :class="{ 'is-manual': manual && !versionId }"
    >
      <div class="download-game-column download-game-browser">
        <PCard v-if="!modify" hide-title class="category-tabs-card">
          <div class="category-tabs" role="tablist">
            <button
              v-for="category in categories"
              :key="category.key"
              type="button"
              class="category-tab"
              :class="{ active: selectedCategory === category.key }"
              role="tab"
              :aria-selected="selectedCategory === category.key"
              @click="selectedCategory = category.key"
            >
              {{ pclText(`Download.Version.Type.${category.label}`) }}
            </button>
          </div>
          <span
            class="category-active-indicator"
            :style="{
              '--active-index': selectedCategoryIndex,
              '--active-column': selectedCategoryIndex % 2,
              '--active-row': Math.floor(selectedCategoryIndex / 2),
            }"
            aria-hidden="true"
          />
        </PCard>
        <PSearchBox
          v-if="!modify"
          v-model="versionQuery"
          class="version-search-card"
          :label="`${pclText(`Download.Version.Type.${selectedCategoryInfo.label}`)} (${visibleVersions.length})`"
          :placeholder="pclText('Download.Version.SearchPlaceholder')"
        />
        <PCard v-if="!modify" hide-title class="version-list-card">
          <div class="version-list">
            <PVirtualList
              :items="visibleVersions"
              :item-key="(version) => version.id"
              v-slot="{ item: version }"
            >
              <PListItem
                :key="version.id"
                :title="version.id"
                :icon="selectedCategoryInfo.icon"
                :subtitle="new Date(version.timestamp).toLocaleString(pclFormatLocale)"
                :selected="version.id === versionId"
                clickable
                @click="() => choose(version.id, selectedCategory)"
              >
                <template #icon-actions>
                  <PVersionActions :version-id="version.id" :manual="manual" />
                </template>
              </PListItem>
            </PVirtualList>
            <p v-if="!visibleVersions.length" class="pcl-empty">
              {{ pclText('Download.Install.State.NoAvailableVersion') }}
            </p>
          </div>
        </PCard>
      </div>

      <div
        v-if="(!manual || versionId) && versionId"
        class="download-game-column download-game-installation"
      >
        <PCard hide-title class="instance-card">
        <div class="instance-name-row">
          <img
            :src="
              instanceIcon(
                versionId,
                selectedPlugins,
                typeof route.query.version_type === 'string' ? route.query.version_type : undefined,
              )
            "
            alt=""
          />
          <PTextBox
            :model-value="instanceName"
            @update:model-value="
              (value) => {
                instanceName = value || ''
                instanceNameEdited = true
              }
            "
            :aria-label="pclText('Proto.Label.InstanceName')"
          />
          <span v-if="modify">{{ pclText('Instance.Install.Action.ModifyLabel') }}</span>
        </div>
        <div v-if="warnings.length" class="install-warnings">
          <PHint
            v-for="warning in warnings"
            :key="warning"
            :severity="warning.endsWith('Api') ? 'error' : 'warning'"
            >{{ pclText(`Download.Install.Warning.${warning}`) }}</PHint
          >
        </div>
        </PCard>
        <div class="loader-grid" role="list">
        <div
          v-for="loader in visibleLoaders"
          :key="`${versionId}-${loader}`"
          class="loader-choice"
          role="listitem"
        >
          <button
            type="button"
            class="loader-choice-card"
            :class="{
              active: activeLoader === loader,
              selected: !!selected[loader],
              disabled: !!loaderConflict(loader, selected, versionId),
            }"
            :disabled="!!loaderConflict(loader, selected, versionId)"
            :aria-pressed="!!selected[loader]"
            @click="activateLoader(loader)"
          >
            <img :src="loaderIcon(loader)" alt="" />
            <span class="loader-choice-copy">
              <strong>{{ pclText(`Common.Installation.${loader}`) }}</strong>
              <small>{{
                loaderConflict(loader, selected, versionId)
                  ? pclText('Download.Install.Compat.IncompatibleWithLoader', [
                      loaderConflict(loader, selected, versionId)!,
                    ])
                  : loaderSummary(loader)
              }}</small>
            </span>
          </button>
        </div>
        </div>
        <PCard v-if="activeLoader && versionId" hide-title class="loader-version-panel">
        <div class="loader-version-heading">
          <strong>{{ pclText(`Common.Installation.${activeLoader}`) }}</strong>
          <PComboBox
            v-if="plugins[activeLoader]?.versions.length"
            v-model="activeLoaderVersion"
            class="loader-version-select"
            :options="activeLoaderOptions"
            :aria-label="pclText(`Common.Installation.${activeLoader}`)"
          />
          <span v-else-if="activeLoaderMessage" class="loader-version-message">
            {{ activeLoaderMessage }}
          </span>
        </div>
        <div
          v-if="plugins[activeLoader]?.loading || plugins[activeLoader]?.error"
          class="loader-version-state"
        >
          <PLoading v-if="plugins[activeLoader]?.loading" :card="false" />
          <PHint v-else-if="plugins[activeLoader]?.error" severity="error"
            >{{ plugins[activeLoader]?.error }}
            <PButton inline @click="loadLoader(activeLoader, versionId, request)">{{
              pclText('Launch.Homepage.LoadFailed.Retry')
            }}</PButton></PHint
          >
        </div>
        </PCard>
      </div>
    </div>
  </section>
</template>
<style scoped>
.download-game-page {
  height: 100%;
  min-height: 0;
  overflow: hidden;
}
.download-game-columns {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 12px 14px;
  height: 100%;
  min-height: 0;
}
.download-game-columns.is-manual {
  grid-template-columns: minmax(0, 1fr);
}
.download-game-column {
  display: flex;
  min-width: 0;
  min-height: 0;
  flex-direction: column;
  gap: 12px;
}
.category-tabs {
  position: relative;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  align-items: center;
  width: 100%;
  min-height: 36px;
}
.category-tab {
  position: relative;
  min-width: 0;
  min-height: 32px;
  border: 0;
  padding: 0 10px;
  color: var(--color-text-grey);
  white-space: nowrap;
  background: transparent;
  font: inherit;
  cursor: pointer;
  transition:
    color 120ms linear,
    background-color 120ms linear,
    box-shadow 120ms linear;
}
.category-tab:hover,
.category-tab:focus-visible {
  color: var(--color-text);
}
.category-tab.active {
  color: var(--color-text);
  font-weight: 600;
}
.category-active-indicator {
  position: absolute;
  left: 0;
  bottom: 0;
  width: 25%;
  height: 4px;
  background: transparent;
  pointer-events: none;
  transform: translateX(calc(var(--active-index) * 100%));
  will-change: transform;
  transition: transform 320ms cubic-bezier(0.22, 0.61, 0.36, 1);
}
.category-active-indicator::after {
  content: '';
  position: absolute;
  left: 50%;
  top: 0;
  width: 24px;
  height: 4px;
  border-radius: 999px;
  background: var(--color-tint);
  transform: translateX(-50%);
}
.version-list-card {
  flex: 1 1 auto;
  min-height: 0;
}
.version-list-card :deep(.mycard),
.version-list-card :deep(.mycard-content) {
  min-height: 0;
  height: 100%;
}
.version-list-card :deep(.mycard) {
  display: flex;
  flex-direction: column;
}
.version-list-card :deep(.mycard-content) {
  flex: 1 1 auto;
}
.version-list {
  min-height: 0;
  height: 100%;
  overflow-y: auto;
  padding: 8px;
}
.loader-grid {
  flex: 1 1 auto;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  grid-auto-rows: minmax(52px, 1fr);
  gap: 8px;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  align-content: stretch;
  padding: 0;
}
.loader-choice {
  position: relative;
  min-width: 0;
}
.loader-choice-card {
  display: flex;
  align-items: center;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: clamp(52px, 8vh, 78px);
  padding: 10px 12px;
  gap: 10px;
  border: 1px solid transparent;
  border-radius: 6px;
  color: var(--color-text);
  background: var(--color-background-soft);
  box-shadow: 0 0 5px rgb(0 0 0 / 7%);
  text-align: left;
  cursor: pointer;
  transition:
    background-color 120ms linear,
    box-shadow 120ms linear,
    opacity 120ms linear;
}
.loader-choice-card:hover {
  box-shadow: 0 0 6px var(--color-tint-shadow);
}
.loader-choice-card.active,
.loader-choice-card.selected {
  background: var(--color-tint-lighter);
  box-shadow: 0 0 6px var(--color-tint-shadow);
}
.loader-choice-card.disabled {
  cursor: not-allowed;
  opacity: 0.48;
  box-shadow: none;
}
.loader-choice-card > img {
  flex: 0 0 24px;
  width: 24px;
  height: 24px;
}
.loader-choice-copy {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  gap: 2px;
}
.loader-choice-copy strong,
.loader-choice-copy small {
  overflow: visible;
  white-space: normal;
  overflow-wrap: anywhere;
  line-height: 1.2;
}
.loader-choice-copy strong {
  font-size: 13px;
}
.loader-choice-copy small {
  color: var(--color-text-grey);
  font-size: 11px;
}
.loader-version-panel {
  min-height: 0;
}
.loader-version-heading {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 20px 12px;
}
.loader-version-heading strong {
  flex: 0 0 auto;
  white-space: nowrap;
}
.loader-version-message {
  min-width: 0;
  flex: 1 1 auto;
  overflow: hidden;
  color: var(--color-text-grey);
  text-overflow: ellipsis;
  white-space: nowrap;
}
.loader-version-state {
  padding: 0 20px 15px;
}
.loader-version-state :deep(.loading-container) {
  min-height: 130px;
  padding: 12px 14px;
  gap: 12px;
}
.loader-version-select {
  flex: 1 1 auto;
  min-width: 0;
  width: 100%;
}
.instance-name-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 15px 16px 15px 13px;
}
.instance-name-row > img {
  width: 32px;
  height: 32px;
}
.instance-name-row > :last-child {
  flex: 1;
}
.install-warnings {
  padding: 0 16px 15px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
@media (max-width: 520px) {
  .category-tabs {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .category-active-indicator {
    width: 50%;
    height: 50%;
    background: transparent;
    transform: translate(calc(var(--active-column) * 100%), calc(var(--active-row) * 100%));
  }
  .category-active-indicator::after {
    top: calc(100% - 4px);
  }
}
@media (prefers-reduced-motion: reduce) {
  .category-active-indicator {
    transition: none;
  }
}
</style>
