<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, shallowRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke, isTauri } from '@tauri-apps/api/core'
import PCard from '@/components/widget/PCard.vue'
import PVirtualList from '@/components/widget/PVirtualList.vue'
import PListItem from '@/components/widget/PListItem.vue'
import PVersionActions from '@/components/widget/PVersionActions.vue'
import PLoaderVersionActions from '@/components/widget/PLoaderVersionActions.vue'
import PButton from '@/components/widget/PButton.vue'
import PTextBox from '@/components/widget/PTextBox.vue'
import PSearchBox from '@/components/widget/PSearchBox.vue'
import PHint from '@/components/widget/PHint.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
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
function loaderSubtitle(version: McPluginReport) {
  if (version.release_time) {
    const date = new Date(version.release_time)
    if (!Number.isNaN(date.getTime()))
      return pclText('Download.Version.ReleaseDate', [date.toLocaleString(pclFormatLocale.value)])
  }
  return version.stable == null
    ? undefined
    : pclText(version.stable ? 'Download.Version.Type.Stable' : 'Download.Version.Type.Preview')
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
const visibleLoaders = computed(() =>
  loaders.filter((loader) => loaderVisible(loader, versionId.value, selected)),
)
watch(
  visibleLoaders,
  (list) => {
    if (!list.includes(activeLoader.value)) activeLoader.value = list[0] || ''
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
const versionQuery = ref('')
const visibleVersions = computed(() => {
  const list = versions.value?.[selectedCategory.value] || []
  const query = versionQuery.value.trim().toLowerCase()
  return query ? list.filter((version) => version.id.toLowerCase().includes(query)) : list
})
const selectedCategoryInfo = computed(() =>
  categories.find((category) => category.key === selectedCategory.value) || categories[0],
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
          query: { version_id: initial.id, version_type: data.latest.release ? 'release' : 'snapshot' },
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
    if (current === request) plugins[name] = { loading: false, versions: list, error: '' }
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
    Object.keys(selected).forEach((key) => delete selected[key])
    Object.keys(fabricApiFiles).forEach((key) => delete fabricApiFiles[key])
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
      <PButton inline @click="load">{{ pclText('Launch.Homepage.LoadFailed.Retry') }}</PButton></PHint
    >
    <div
      v-else-if="versions"
      class="download-game-columns"
      :class="{ 'is-manual': manual && !versionId }"
    >
      <div v-if="!modify" class="version-browser">
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
        <PSearchBox
          v-model="versionQuery"
          :placeholder="pclText('Download.Version.SearchPlaceholder')"
        />
        <PCard
          :title="`${pclText(`Download.Version.Type.${selectedCategoryInfo.label}`)} (${visibleVersions.length})`"
          class="version-list-card"
        >
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

      <div v-if="!manual || versionId" class="installation-panel">
        <template v-if="versionId">
          <PCard hide-title>
            <div class="instance-name-row">
              <button
                v-if="!modify"
                type="button"
                class="pcl-icon-button install-back"
                @click="() => router.replace('/download/game')"
                :aria-label="pclText('Common.Action.Back')"
              >
                <PSvgIcon name="lucide/arrow-left" :size="16" />
              </button>
              <img
                :src="instanceIcon(
                  versionId,
                  selectedPlugins,
                  typeof route.query.version_type === 'string' ? route.query.version_type : undefined,
                )"
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
                :aria-pressed="activeLoader === loader"
                @click="activeLoader = loader"
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
                <PSvgIcon name="lucide/chevron-right" :size="16" />
              </button>
              <button
                v-if="selected[loader]"
                class="pcl-icon-button loader-clear"
                type="button"
                :title="pclText('Proto.Action.ClearSelection')"
                :aria-label="pclText('Proto.Action.ClearSelection')"
                @click="chooseLoader(loader, '')"
              >
                <PSvgIcon name="lucide/x" :size="15" />
              </button>
            </div>
          </div>
          <PCard
            v-if="activeLoader"
            hide-title
            class="loader-version-panel"
          >
            <div class="loader-version-heading">
              <strong>{{ pclText(`Common.Installation.${activeLoader}`) }}</strong>
              <span>{{ pclText('Proto.Download.SelectVersion') }}</span>
            </div>
            <div class="version-list">
              <PLoading v-if="plugins[activeLoader]?.loading" :card="false" />
              <PHint v-else-if="plugins[activeLoader]?.error" severity="error"
                >{{ plugins[activeLoader]?.error }}
                <PButton inline @click="loadLoader(activeLoader, versionId, request)">{{
                  pclText('Launch.Homepage.LoadFailed.Retry')
                }}</PButton></PHint
              >
              <template v-else-if="plugins[activeLoader]?.versions.length">
                <PVirtualList
                  :items="plugins[activeLoader]!.versions"
                  :item-key="(version) => version.version"
                  v-slot="{ item: version }"
                >
                  <PListItem
                    :key="version.version"
                    :title="version.version"
                    :icon="loaderIcon(activeLoader)"
                    :subtitle="loaderSubtitle(version)"
                    :selected="selected[activeLoader] === version.version"
                    clickable
                    @click="() => chooseLoader(activeLoader, version.version)"
                    ><template v-if="activeLoader !== 'FabricApi'" #icon-actions
                      ><PLoaderVersionActions
                        :loader="activeLoader"
                        :game="versionId"
                        :version="version.version" /></template
                  ></PListItem>
                </PVirtualList>
              </template>
              <p v-else class="pcl-empty">
                {{
                  ['Forge', 'NeoForge', 'Fabric', 'LegacyFabric', 'FabricApi'].includes(activeLoader)
                    ? pclText('Download.Install.State.NoAvailableVersion')
                    : pclText('Proto.NotImplemented.Loader')
                }}
              </p>
            </div>
          </PCard>
          <PButton v-if="!manual" class="install-action" type="tint" @click="install">
            {{ modify ? pclText('Instance.Install.Action.StartModify') : pclText('Download.Install.StartDownload') }}
          </PButton>
        </template>
      </div>
    </div>
  </section>
</template>
<style scoped>
.download-game-columns {
  display: grid;
  grid-template-columns: minmax(0, 1.7fr) minmax(280px, 1fr);
  gap: 14px;
  align-items: start;
}
.version-browser,
.installation-panel {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.category-tabs {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  width: fit-content;
  max-width: 100%;
  min-height: 40px;
  padding: 4px;
  border-radius: 999px;
  background: var(--control-background);
}
.category-tab {
  min-width: 0;
  min-height: 32px;
  border: 0;
  border-radius: 999px;
  padding: 0 13px;
  color: var(--color-text-grey);
  white-space: nowrap;
  background: transparent;
  font: inherit;
  cursor: pointer;
  transition: color 120ms linear, background-color 120ms linear, box-shadow 120ms linear;
}
.category-tab:hover,
.category-tab:focus-visible {
  color: var(--color-text);
}
.category-tab.active {
  color: var(--color-text);
  background: var(--color-background-soft);
  box-shadow: 0 1px 4px rgb(0 0 0 / 8%);
  font-weight: 600;
}
.version-list-card :deep(.mycard-title > p) {
  font-size: 14px;
}
.version-list {
  padding: 0 18px 12px 20px;
}
.loader-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}
.loader-choice {
  position: relative;
  min-width: 0;
}
.loader-choice > .loader-clear {
  position: absolute;
  top: 4px;
  right: 4px;
  z-index: 1;
  width: 22px;
  height: 22px;
  color: var(--color-text-grey);
}
.loader-choice-card {
  display: flex;
  align-items: center;
  width: 100%;
  height: 78px;
  min-width: 0;
  padding: 12px 28px 12px 12px;
  gap: 10px;
  border: 1px solid transparent;
  border-radius: 6px;
  color: var(--color-text);
  background: var(--color-background-soft);
  box-shadow: 0 0 5px rgb(0 0 0 / 7%);
  text-align: left;
  cursor: pointer;
  transition: border-color 120ms linear, box-shadow 120ms linear;
}
.loader-choice-card:hover,
.loader-choice-card:focus-visible,
.loader-choice-card.active {
  border-color: var(--color-tint);
  box-shadow: 0 0 6px var(--color-tint-shadow);
}
.loader-choice-card.selected {
  background: var(--color-tint-lighter);
}
.loader-choice-card.disabled {
  cursor: not-allowed;
  opacity: 0.55;
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
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.loader-choice-copy strong {
  font-size: 13px;
}
.loader-choice-copy small {
  color: var(--color-text-grey);
  font-size: 11px;
}
.loader-choice-card > :last-child {
  flex: 0 0 auto;
  color: var(--color-text-grey);
}
.loader-version-panel {
  min-height: 150px;
}
.loader-version-heading {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 10px;
  padding: 14px 20px 8px;
}
.loader-version-heading span {
  color: var(--color-text-grey);
  font-size: 12px;
}
.loader-choice:hover .loader-clear:not(:disabled),
.loader-clear:not(:disabled):focus-visible {
  color: var(--color-tint);
  transition-duration: 120ms;
}
.loader-clear {
  color: var(--color-text-grey);
  transition: color 150ms linear;
}
.loader-clear:not(:disabled):active {
  transform: none;
}
.loader-clear:focus-visible {
  outline: 2px solid var(--color-text-grey);
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
.install-back {
  flex: 0 0 26px;
  height: 26px;
  width: 26px;
  color: var(--color-text-grey);
}
.install-back:focus-visible {
  outline: 2px solid var(--color-tint);
  outline-offset: 2px;
}
.install-warnings {
  padding: 0 16px 15px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.install-action {
  width: 100%;
}
@media (max-width: 520px) {
  .category-tabs {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>