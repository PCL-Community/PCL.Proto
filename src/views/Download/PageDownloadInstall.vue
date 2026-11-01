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
import { useTaskManager } from '@/stores/task'
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
import { FloatButtonType, useFloatButton } from '@/composables/useFloatButton'

const props = defineProps<{ manual?: boolean; modify?: boolean }>()
const route = useRoute()
const router = useRouter()
const text = pclText
const versions = shallowRef<IVersionShow>()
const loading = ref(false)
const error = ref('')
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
  loaderCards[loader]?.collapse()
  if (!instanceNameEdited.value) instanceName.value = instanceDefaultName(versionId.value, selected)
}
const warnings = computed(() => installWarnings(selected, versionId.value))
function loaderSubtitle(version: McPluginReport) {
  if (version.release_time) {
    const date = new Date(version.release_time)
    if (!Number.isNaN(date.getTime()))
      return text('Download.Version.ReleaseDate', [date.toLocaleString(pclFormatLocale.value)])
  }
  return version.stable == null
    ? undefined
    : text(version.stable ? 'Download.Version.Type.Stable' : 'Download.Version.Type.Preview')
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
    list.push({ ...data.latest.snapshot, type: 'snapshot', label: 'Development' })
  return list
})
async function load() {
  loading.value = true
  error.value = ''
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
  void router.push({ path: '/download/game/inner', query: { version_id: id, version_type: type } })
}
let request = 0
async function loadLoader(name: string, id: string, current: number) {
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
    const current = ++request
    if (!id) return
    await Promise.all(
      ['Forge', 'Fabric', 'LegacyFabric'].map((name) => loadLoader(name, id, current)),
    )
  },
  { immediate: true },
)
async function install() {
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
      ([name, value]) => !['Fabric', 'LegacyFabric'].includes(name) && !!value,
    )
  ) {
    sideTip.show(pclText('Proto.NotImplemented.LoaderInstall'), 'warn')
    return
  }
  installing.value = true
  try {
    void useTaskManager()
      .startDownloadMCVersion(
        versionId.value,
        instanceName.value.trim(),
        selected.Fabric
          ? { family: 'fabric', version: selected.Fabric }
          : selected.LegacyFabric
            ? { family: 'legacyfabric', version: selected.LegacyFabric }
            : undefined,
      )
      .catch((err) => sideTip.show(pclText('Proto.Error.Download', [String(err)]), 'warn'))
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
  <section class="pcl-page install-page">
    <p v-if="modify && !versionId" class="pcl-empty">{{ pclText('Proto.Instance.SelectFirst') }}</p>
    <template v-else-if="!versionId">
      <PLoading v-if="loading" :loading-text="text('Download.Version.LoadingList')" />
      <PHint v-else-if="error" severity="error"
        >{{ error }}
        <PButton inline @click="load">{{
          pclText('Launch.Homepage.LoadFailed.Retry')
        }}</PButton></PHint
      >
      <template v-else-if="versions">
        <PCard :title="text('Download.Version.Latest.Title')" class="latest-card">
          <div class="version-list">
            <PListItem
              v-for="version in latest"
              :key="version.id"
              :title="version.id"
              :icon="version.type === 'release' ? showIconPath.grass : showIconPath.command"
              :subtitle="
                text(`Download.Version.Latest.${version.label}`, [
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
            :title="`${text(`Download.Version.Type.${category.label}`)} (${versions[category.key].length})`"
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
            class="install-back"
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
          <span v-if="modify">{{ text('Instance.Install.Action.ModifyLabel') }}</span>
        </div>
        <div v-if="warnings.length" class="install-warnings">
          <PHint
            v-for="warning in warnings"
            :key="warning"
            :severity="warning.endsWith('Api') ? 'error' : 'warning'"
            >{{ text(`Download.Install.Warning.${warning}`) }}</PHint
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
        :title="text(`Common.Installation.${loader}`)"
        :disabled="!!loaderConflict(loader, selected, versionId)"
        default-fold-status="fold"
        fold-arrow="right"
      >
        <template #description
          ><span class="loader-status" :class="{ 'is-selected': selected[loader] }"
            ><img v-if="selected[loader]" :src="loaderIcon(loader)" alt="" />{{
              selected[loader] ||
              (loaderConflict(loader, selected, versionId)
                ? text('Download.Install.Compat.IncompatibleWithLoader', [
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
            class="loader-clear"
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
              text('Launch.Homepage.LoadFailed.Retry')
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
                ><template #icon-actions
                  ><PLoaderVersionActions
                    :loader="loader"
                    :game="versionId"
                    :version="version.version" /></template
              ></PListItem>
            </PVirtualList>
          </template>
          <p v-else class="pcl-empty">
            {{
              ['Forge', 'Fabric', 'LegacyFabric'].includes(loader)
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
.install-page {
  padding: 25px 25px 80px;
  display: flex;
  flex-direction: column;
  gap: 15px;
}
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
  width: 26px;
  height: 26px;
  padding: 5px;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: 50%;
  background: transparent;
  color: rgba(0, 0, 0, 0.62745);
  opacity: 0.6;
  cursor: pointer;
  transition: color 150ms linear;
}
.install-back:hover,
.install-back:focus-visible {
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
:global(.dark) .install-back:hover,
:global(.dark) .install-back:focus-visible {
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
}
.loader-status {
  color: var(--color-text-grey);
  font-size: 12px;
  display: inline-flex;
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
.loader-card:hover .loader-clear {
  color: var(--color-tint);
  transition-duration: 120ms;
}
.loader-clear {
  display: grid;
  place-items: center;
  width: 25px;
  height: 25px;
  padding: 5px;
  border: 0;
  background: transparent;
  color: var(--color-text-grey);
  cursor: pointer;
  /* CE MyIconButton：移出 150 ms、指向 120 ms。 */
  transition: color 150ms linear;
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
