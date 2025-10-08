<script setup lang="ts">
import PCard from '@/components/widget/PCard.vue'
import PInput from '@/components/widget/PInput.vue'
import { showIconPath, type pluginType } from '@/util/gameInfo'
import { info } from '@tauri-apps/plugin-log'
import { onMounted, onUnmounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ArrowLeft from '@/assets/icons/ArrowLeft.svg'
import ModifyCard from '@/components/widget/ModifyCard.vue'
import { FloatButtonType, useFloatButton } from '@/composables/useFloatButton'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { useTaskManager } from '@/stores/task'
import type { VersionDetails } from '@/api/gameVersions'

const router = useRouter()
const version_id = useRoute().query.version as string
const instance_name = ref(version_id)
// const pluginTypes = Object.keys(pluginShowText) as pluginType[]
const { floatButtonState, setFloatButton } = useFloatButton()
var unlistenButton: any
const taskManager = useTaskManager()
// let pluginVersions = new Map<pluginType, Array<string>>()
const pluginVersions = ref({} as Record<pluginType, string[]>)
const pluginSelectState = ref({} as Record<pluginType, string>)
const arrowLeftClicked = () => {
  router.back()
}

onMounted(async () => {
  floatButtonState.visible = true
  setFloatButton(FloatButtonType.DownloadGame)
  unlistenButton = await listen('float-button-click', (event) => {
    if (event.payload === FloatButtonType.DownloadGame) {
      info(`download game: ${version_id}`)
      taskManager.startDownloadMCVersion(version_id)
      setFloatButton(FloatButtonType.TaskManage)
      router.back()
    }
  })
  // get version info from backend
  let versionDetails = await invoke<VersionDetails>('handle_clicked_on_version', {
    id: version_id,
  })
  info(`got version info: ${versionDetails.id}`)
  pluginVersions.value.forge = await invoke<string[]>('get_forge_versions', { version_id })
  console.log(pluginVersions.value.forge)
})

onUnmounted(() => {
  if (floatButtonState.type === FloatButtonType.DownloadGame) {
    floatButtonState.visible = false
  }
  unlistenButton?.()
})

function onSelect(plugin: pluginType, versionId: string) {
  pluginSelectState.value[plugin] = versionId
  console.log(pluginSelectState.value)
}
</script>

<template>
  <PCard hide-title>
    <i class="arrow-left button-animated" @click="arrowLeftClicked"> <ArrowLeft /></i>
    <img :src="showIconPath['vanilla']" />
    <PInput v-model="instance_name" />
  </PCard>
  <ModifyCard :plugin="'vanilla'" :versions="[version_id]" :is-loading="false" />
  <ModifyCard
    :plugin="'forge'"
    :is-loading="pluginVersions['forge'] == undefined"
    :versions="pluginVersions['forge']"
    @select-version="onSelect"
  />
</template>

<style lang="css" scoped>
.arrow-left {
  margin-inline: 6px;
  color: var(--vt-c-gray);
  cursor: pointer;
}

img {
  width: 30px;
  height: 30px;
}

/* set the title flex row */
:deep(section.mycard-content:nth-child(1)) {
  flex-direction: row;
  align-items: center;
  gap: 10px;
}

:deep(.input-wrapper) {
  flex: 1;
}
</style>
