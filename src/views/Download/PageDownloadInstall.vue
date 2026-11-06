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
const loaderCards: Record<string, InstanceType<typeof PCard> | null> = {}
function chooseLoader(loader: string, version: string) {
  selectLoader(selected, loader, version)
  if (loader === 'FabricApi') fabricApiAutoSelected = true
  if (!selected.Fabric) fabricApiAutoSelected = false
  if (loader === 'Fabric' && version && !fabricApiAutoSelected && plugins.FabricApi?.versions[0]) {
    fabricApiAutoSelected = true
    chooseLoader('FabricApi', plugins.FabricApi.versions[0].version)
  }
  loaderCards[loader]?.collapse()
  if (!instanceNameEdited.value) instanceName.value = instanceDefaultName(versionId.value, selected)
}
const warnings = computed(() => installWarnings(selected, versionId.value))
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
const categories = [
  { key: 'release', label: 'Release', icon: showIconPath.grass },
  { key: 'snapshot', label: 'Development', icon: showIconPath.command },
  { key: 'old', label: 'BeforeRelease', icon: showIconPath.stone },
  { key: 'fool', label: 'AprilFools', icon: showIconPath.gold },
] as const
const latest = computed(() => {
  const data = versions.value
  if (!data) return []
  const list = data.latest.release
    ? [{ ...data.latest.release, type: 'release', label: 'Release' }]
    : []
  if (
    data.latest.snapshot &&
    (!data.latest.release || data.latest.snapshot.timestamp > data.latest.release.timestamp)
  )
    list.push({
      ...data.latest.snapshot,
      type: 'snapshot',
      label: 'Development',
    })
  return list
})
async function load() {
  loading.value = true
  error.value = null
  try {
    versions.value = await getMinecraftVersions()
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
  void router.push({
    path: '/download/game/inner',
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
  <section class="pcl-page">
    <p v-if="modify && !versionId" class="pcl-empty">
      {{ pclText('Proto.Instance.SelectFirst') }}
    </p>
    <template v-else-if="!versionId">
      <PLoading v-if="loading" :loading-text="pclText('Download.Version.LoadingList')" />
      <PHint v-else-if="error" severity="error"
        >{{ error }}
        <PButton inline @click="load">{{
          pclText('Launch.Homepage.LoadFailed.Retry')
        }}</PButton></PHint
      >
      <template v-else-if="versions">
        <PCard :title="pclText('Download.Version.Latest.Title')" class="latest-card">
          <div class="version-list">
            <PListItem
              v-for="version in latest"
              :key="version.id"
              :title="version.id"
              :icon="version.type === 'release' ? showIconPath.grass : showIconPath.command"
              :subtitle="
                pclText(`Download.Version.Latest.${version.label}`, [
                  new Date(version.timestamp).toLocaleString(pclFormatLocale),
                ])
              "
              clickable
              @click="() => choose(version.id, version.type)"
              ><template #icon-actions
                ><PVersionActions :version-id="version.id" :manual="manual" /></template
            ></PListItem>
          </div>
        </PCard>
        <template v-for="category in categories" :key="category.key">
          <PCard
            v-if="versions[category.key].length"
            :title="`${pclText(`Download.Version.Type.${category.label}`)} (${versions[category.key].length})`"
            default-fold-status="fold"
          >
            <div class="version-list">
              <PVirtualList
                :items="versions[category.key]"
                :item-key="(version) => version.id"
                v-slot="{ item: version }"
              >
                <PListItem
                  :key="version.id"
                  :title="version.id"
                  :icon="category.icon"
                  :subtitle="new Date(version.timestamp).toLocaleString(pclFormatLocale)"
                  clickable
                  @click="() => choose(version.id, category.key)"
                  ><template #icon-actions
                    ><PVersionActions :version-id="version.id" :manual="manual" /></template
                ></PListItem>
              </PVirtualList>
            </div>
          </PCard>
        </template>
      </template>
    </template>
    <template v-else>
      <PCard hide-title>
        <div class="instance-name-row">
          <button
            type="button"
            class="pcl-icon-button install-back"
            @click="() => router.push('/download/game')"
            :aria-label="pclText('Common.Action.Back')"
          >
            <PSvgIcon name="lucide/arrow-left" :size="16" />
          </button>
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
      <PCard
        v-for="loader in visibleLoaders"
        class="loader-card"
        :key="`${versionId}-${loader}`"
        :ref="
          (card) => {
            loaderCards[loader] = card as InstanceType<typeof PCard> | null
          }
        "
        :title="pclText(`Common.Installation.${loader}`)"
        :disabled="!!loaderConflict(loader, selected, versionId)"
        default-fold-status="fold"
        fold-arrow="right"
      >
        <template #description
          ><span class="loader-status" :class="{ 'is-selected': selected[loader] }"
            ><img v-if="selected[loader]" :src="loaderIcon(loader)" alt="" />{{
              selected[loader] ||
              (loaderConflict(loader, selected, versionId)
                ? pclText('Download.Install.Compat.IncompatibleWithLoader', [
                    loaderConflict(loader, selected, versionId)!,
                  ])
                : '') ||
              (plugins[loader]?.loading
                ? pclText('Proto.Download.Loading')
                : pclText('Proto.Download.SelectVersion'))
            }}</span
          ></template
        >
        <template #header-actions>
          <button
            v-if="selected[loader]"
            class="pcl-icon-button loader-clear"
            type="button"
            :title="pclText('Proto.Action.ClearSelection')"
            :aria-label="pclText('Proto.Action.ClearSelection')"
            @click.stop="chooseLoader(loader, '')"
          >
            <PSvgIcon name="lucide/x" :size="15" />
          </button>
        </template>
        <div class="version-list">
          <PLoading v-if="plugins[loader]?.loading" :card="false" />
          <PHint v-else-if="plugins[loader]?.error" severity="error"
            >{{ plugins[loader]?.error }}
            <PButton inline @click="loadLoader(loader, versionId, request)">{{
              pclText('Launch.Homepage.LoadFailed.Retry')
            }}</PButton></PHint
          >
          <template v-else-if="plugins[loader]?.versions.length">
            <PVirtualList
              :items="plugins[loader]!.versions"
              :item-key="(version) => version.version"
              v-slot="{ item: version }"
            >
              <PListItem
                :key="version.version"
                :title="version.version"
                :icon="loaderIcon(loader)"
                :subtitle="loaderSubtitle(version)"
                clickable
                @click="() => chooseLoader(loader, version.version)"
                ><template v-if="loader !== 'FabricApi'" #icon-actions
                  ><PLoaderVersionActions
                    :loader="loader"
                    :game="versionId"
                    :version="version.version" /></template
              ></PListItem>
            </PVirtualList>
          </template>
          <p v-else class="pcl-empty">
            {{
              ['Forge', 'NeoForge', 'Fabric', 'LegacyFabric', 'FabricApi'].includes(loader)
                ? pclText('Download.Install.State.NoAvailableVersion')
                : pclText('Proto.NotImplemented.Loader')
            }}
          </p>
        </div>
      </PCard>
    </template>
  </section>
</template>
<style scoped>
.version-list {
  padding: 0 18px 12px 20px;
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
.install-back {
  flex: 0 0 26px;
  height: 26px;
  width: 26px;
  color: rgba(0, 0, 0, 0.62745);
  opacity: 0.6;
  transition: color 150ms linear;
}
.install-back:not(:disabled):active {
  transform: none;
}
.install-back:not(:disabled):hover,
.install-back:not(:disabled):focus-visible {
  color: rgba(0, 0, 0, 0.90196);
  transition-duration: 120ms;
}
.install-back:focus-visible {
  outline: 2px solid var(--color-tint);
  outline-offset: 2px;
}
.install-back :deep(.pcl-icon) {
  transition: transform 250ms cubic-bezier(0.34, 2, 0.64, 1);
}
.install-back:active :deep(.pcl-icon) {
  transform: scale(0.8);
  transition: transform 400ms cubic-bezier(0.25, 1, 0.5, 1);
}
:global(.dark) .install-back {
  color: rgba(255, 255, 255, 0.62745);
}
:global(.dark) .install-back:not(:disabled):hover,
:global(.dark) .install-back:not(:disabled):focus-visible {
  color: rgba(255, 255, 255, 0.90196);
}
.instance-name-row > :last-child {
  flex: 1;
}
.loader-card :deep(.mycard-title > p) {
  flex: 0 0 min(117px, 30%);
}
.loader-card :deep(.mycard-title > .description) {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
}
.loader-status {
  color: var(--color-text-grey);
  font-size: 12px;
  line-height: 1;
  display: flex;
  align-items: center;
  gap: 7px;
}
.loader-status.is-selected {
  color: var(--color-text);
}
.loader-status img {
  width: 18px;
  height: 18px;
}
.loader-card :deep(.mycard-title-actions > i) {
  color: var(--color-text-grey);
}
.loader-card:hover :deep(.mycard-title-actions > i) {
  color: var(--color-tint);
}
.loader-card:hover .loader-clear:not(:disabled),
.loader-clear:not(:disabled):focus-visible {
  color: var(--color-tint);
  transition-duration: 120ms;
}
.loader-clear {
  color: var(--color-text-grey);
  /* CE MyIconButton：移出 150 ms、指向 120 ms。 */
  transition: color 150ms linear;
}
.loader-clear:not(:disabled):active {
  transform: none;
}
.loader-clear:focus-visible {
  outline: 2px solid var(--color-text-grey);
}
.install-warnings {
  padding: 0 16px 15px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
</style>
