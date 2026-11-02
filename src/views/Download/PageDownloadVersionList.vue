<script setup lang="ts">
import { computed, ref, shallowRef, watch, onMounted } from 'vue'
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
const props = defineProps<{ loader: string; website: string; supported?: boolean }>()
const localeName = computed(() => (props.loader === 'OptiFine' ? 'Optifine' : props.loader))
const fabricFamily = computed(() => ['Fabric', 'LegacyFabric'].includes(props.loader))
const text = pclText
const minecraft = ref('')
const minecraftOptions = ref<{ key: string; text: string }[]>([])
const versions = shallowRef<McPluginReport[]>([])
const error = ref('')
const loading = ref(false)
let request = 0
async function load() {
  if (!props.supported || (!fabricFamily.value && !minecraft.value)) return
  const current = ++request
  loading.value = true
  error.value = ''
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
  loading.value = true
  try {
    const data = await getMinecraftVersions()
    minecraftOptions.value = [...data.release, ...data.snapshot, ...data.old].map((version) => ({
      key: version.id,
      text: version.id,
    }))
    minecraft.value = data.latest.release?.id || minecraftOptions.value[0]?.key || ''
    if (!minecraft.value) loading.value = false
  } catch (err) {
    error.value = pclText('Proto.Error.GameVersion', [String(err)])
    loading.value = false
  }
}
onMounted(initialize)
function retry() {
  return fabricFamily.value || minecraft.value ? load() : initialize()
}
watch(minecraft, load)
async function selectVersion(version: McPluginReport) {
  if (!fabricFamily.value) return sideTip.show(pclText('Proto.NotImplemented.Installer'))
  try {
    const destination = await save({
      title: text('Download.Version.SelectSaveLocation'),
      defaultPath: `fabric-installer-${version.version}.jar`,
      filters: [{ name: 'JAR', extensions: ['jar'] }],
    })
    if (!destination) return
    const name = text(`Minecraft.Download.Stage.${props.loader}InstallerDownload`, [
      version.version,
    ])
    const download = useTaskManager().downloadFabricInstaller(
      name,
      props.loader.toLowerCase(),
      version.version,
      destination,
    )
    sideTip.show(text('Download.Comp.QuickDownload.Hint.DownloadStarted', [version.version]))
    await download
    sideTip.show(text('Proto.Download.Saved', [destination]), 'success')
  } catch (error) {
    if (!isTaskCancelled(error)) sideTip.show(text('Proto.Error.Download', [String(error)]), 'warn')
  }
}
</script>
<template>
  <section class="pcl-page loader-page">
    <PCard :title="text(`Download.Version.${localeName}.Intro.Title`)"
      ><div class="intro">
        <p>{{ text(`Download.Version.${localeName}.Intro.Description`) }}</p>
        <PButton type="tint" @click="() => openExternal(website)">{{
          text('Download.Version.OpenWebsite')
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
          :loading-text="text(`Download.Version.${localeName}.LoadingList`)" /></PCard
      ><PHint v-else-if="error" severity="error"
        >{{ error
        }}<PButton inline @click="retry">{{
          pclText('Launch.Homepage.LoadFailed.Retry')
        }}</PButton></PHint
      ><PCard
        v-else
        :title="
          fabricFamily
            ? text('Download.Version.VersionListCount', [versions.length])
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
              :subtitle="version.stable ? text('Download.Version.Type.Stable') : undefined"
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
      text('Proto.Download.InstallerUnavailable', [loader])
    }}</PHint>
  </section>
</template>
<style scoped>
.loader-page {
  padding: 25px;
  display: flex;
  flex-direction: column;
  gap: 15px;
}
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
