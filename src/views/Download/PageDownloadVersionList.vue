<script setup lang="ts">
import { computed, ref, shallowRef, watch } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { getMinecraftVersions } from '@/api/gameVersions'
import type { McPluginReport } from '@/types/mcPlugin'
import PCard from '@/components/widget/PCard.vue'
import PVirtualList from '@/components/widget/PVirtualList.vue'
import PListItem from '@/components/widget/PListItem.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PButton from '@/components/widget/PButton.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PHint from '@/components/widget/PHint.vue'
import { pclText } from '@/api/pclLocale'
import sideTip from '@/composables/sideTip'
import { loaderIcon } from '@/types/gameInfo'
import { openExternal } from '@/util/externalLink'
import { isTaskCancelled, useTaskManager } from '@/stores/task'
import { downloadForgelike } from '@/api/forgelikeDownload'
const props = defineProps<{
  loader: string
  website: string
  supported?: boolean
}>()
const localeName = computed(() => (props.loader === 'OptiFine' ? 'Optifine' : props.loader))
const fabricFamily = computed(() => ['Fabric', 'LegacyFabric'].includes(props.loader))
const minecraft = ref<string>()
const minecraftOptions = ref<{ key: string; text: string }[]>([])
const versions = shallowRef<McPluginReport[]>([])
const error = ref<string | null>(null)
const loading = ref(false)
let request = 0
async function load() {
  if (!props.supported || (!fabricFamily.value && !minecraft.value)) return
  const current = ++request
  loading.value = true
  error.value = null
  try {
    if (!isTauri()) throw new Error(pclText('Proto.Desktop.Loaders'))
    const data = fabricFamily.value
      ? await invoke<McPluginReport[]>('get_fabric_installers', {
          family: props.loader.toLowerCase(),
        })
      : await invoke<McPluginReport[]>('get_plugin_versions', {
          plugin_type: props.loader.toLowerCase(),
          mc_version: minecraft.value,
        })
    if (current === request) versions.value = data
  } catch (err) {
    if (current === request) error.value = pclText('Proto.Error.Versions', [String(err)])
  } finally {
    if (current === request) loading.value = false
  }
}
async function initialize() {
  if (!props.supported) return
  if (fabricFamily.value) return load()
  const current = ++request
  loading.value = true
  try {
    const data = await getMinecraftVersions()
    if (current !== request) return
    minecraftOptions.value = [...data.release, ...data.snapshot, ...data.old].map((version) => ({
      key: version.id,
      text: version.id,
    }))
    minecraft.value = data.latest.release?.id || minecraftOptions.value[0]?.key
    if (!minecraft.value) loading.value = false
  } catch (err) {
    if (current !== request) return
    error.value = pclText('Proto.Error.GameVersion', [String(err)])
    loading.value = false
  }
}
watch(
  () => props.loader,
  () => {
    request++
    minecraft.value = undefined
    minecraftOptions.value = []
    versions.value = []
    error.value = null
    loading.value = false
    void initialize()
  },
  { immediate: true },
)
function retry() {
  return fabricFamily.value || minecraft.value ? load() : initialize()
}
watch(minecraft, load)
async function selectVersion(version: McPluginReport) {
  try {
    if (!fabricFamily.value) {
      if (!minecraft.value) return
      const destination = await downloadForgelike(props.loader, minecraft.value, version.version)
      if (destination) sideTip.show(pclText('Proto.Download.Saved', [destination]), 'success')
      return
    }
    const destination = await save({
      title: pclText('Download.Version.SelectSaveLocation'),
      defaultPath: `fabric-installer-${version.version}.jar`,
      filters: [{ name: 'JAR', extensions: ['jar'] }],
    })
    if (!destination) return
    const name = pclText(`Minecraft.Download.Stage.${props.loader}InstallerDownload`, [
      version.version,
    ])
    const download = useTaskManager().downloadFabricInstaller(
      name,
      props.loader.toLowerCase(),
      version.version,
      destination,
    )
    sideTip.show(pclText('Download.Comp.QuickDownload.Hint.DownloadStarted', [version.version]))
    await download
    sideTip.show(pclText('Proto.Download.Saved', [destination]), 'success')
  } catch (error) {
    if (!isTaskCancelled(error))
      sideTip.show(pclText('Proto.Error.Download', [String(error)]), 'warn')
  }
}
</script>
<template>
  <section class="pcl-page">
    <PCard :title="pclText(`Download.Version.${localeName}.Intro.Title`)"
      ><div class="intro">
        <p>{{ pclText(`Download.Version.${localeName}.Intro.Description`) }}</p>
        <PButton type="tint" @click="() => openExternal(website)">{{
          pclText('Download.Version.OpenWebsite')
        }}</PButton>
      </div></PCard
    ><template v-if="supported"
      ><PComboBox
        v-if="minecraftOptions.length"
        v-model="minecraft"
        :options="minecraftOptions"
        :aria-label="pclText('Proto.Label.MinecraftVersion')"
      /><PCard v-if="loading" hide-title
        ><PLoading
          :card="false"
          :loading-text="pclText(`Download.Version.${localeName}.LoadingList`)" /></PCard
      ><PHint v-else-if="error" severity="error"
        >{{ error
        }}<PButton inline @click="retry">{{
          pclText('Launch.Homepage.LoadFailed.Retry')
        }}</PButton></PHint
      ><PCard
        v-else
        :title="
          fabricFamily
            ? pclText('Download.Version.VersionListCount', [versions.length])
            : minecraft || loader
        "
        ><div class="version-list">
          <PVirtualList
            :items="versions"
            :item-key="(version) => version.version"
            v-slot="{ item: version }"
          >
            <PListItem
              :key="version.version"
              :title="version.version"
              :icon="loaderIcon(loader)"
              :subtitle="version.stable ? pclText('Download.Version.Type.Stable') : undefined"
              clickable
              @click="() => selectVersion(version)"
            />
          </PVirtualList>
          <p v-if="!versions.length" class="pcl-empty">
            {{ pclText('Download.Install.State.NoAvailableVersion') }}
          </p>
        </div></PCard
      ></template
    ><PHint v-else severity="info">{{
      pclText('Proto.Download.InstallerUnavailable', [loader])
    }}</PHint>
  </section>
</template>
<style scoped>
.intro {
  padding: 0 25px 20px;
}
.intro > p {
  white-space: pre-line;
  margin-bottom: 11px;
}
.version-list {
  padding: 0 20px 15px;
}
</style>
