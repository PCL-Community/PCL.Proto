<script setup lang="ts">
import PCard from '@/components/widget/PCard.vue'
import PInput from '@/components/widget/PInput.vue'
import { gameInfoIcon, showIconPath, type pluginType } from '@/util/gameInfo'
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
import type { McPluginReport } from '@/types/mcPlugin'

const router = useRouter()

const { version_id, version_type } = useRoute().query as {
  version_id: string
  version_type: 'release' | 'snapshot' | 'old'
}

const instance_name = ref(version_id)
const { floatButtonState, setFloatButton } = useFloatButton()
var unlistenButton: any
const taskManager = useTaskManager()
const arrowLeftClicked = () => {
  router.back()
}

// defaults are all undefined
const pluginVersions = ref({} as { [K in pluginType]: McPluginReport[] | null })
const pluginSelectState = ref({} as { [K in pluginType]: string })
let avaliablePlugins = ['forge', 'fabric'] satisfies pluginType[]

onMounted(async () => {
  floatButtonState.visible = true
  setFloatButton(FloatButtonType.DownloadGame)
  unlistenButton = await listen('float-button-click', (event) => {
    if (event.payload === FloatButtonType.DownloadGame) {
      info(`download game: ${version_id}`)
      taskManager.startDownloadMCVersion(version_id, instance_name.value)
      setFloatButton(FloatButtonType.TaskManage)
      router.back()
    }
  })
  // get version info from backend
  let versionDetails = await invoke<VersionDetails>('handle_clicked_on_version', {
    id: version_id,
  })
  info(`got version info: ${versionDetails.id}`)
  await Promise.all(
    avaliablePlugins.map(async (plugin) => {
      try {
        pluginVersions.value[plugin] = await invoke<McPluginReport[]>('get_plugin_versions', {
          plugin_type: plugin,
          mc_version: version_id,
        })
      } catch (err) {
        console.error(`Failed to get versions for ${plugin}:`, err)
        pluginVersions.value[plugin] = null
      }
    }),
  )
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
    <img :src="showIconPath[gameInfoIcon[version_type]]" />
    <PInput v-model="instance_name" />
  </PCard>
  <ModifyCard
    :plugin="'vanilla'"
    :versions="[{ version: version_id, stable: null }]"
    :is-loading="false"
    :icon-type="gameInfoIcon[version_type]"
  />
  <ModifyCard
    v-for="plugin in avaliablePlugins"
    :plugin
    :is-loading="pluginVersions[plugin] === undefined"
    :versions="pluginVersions[plugin]"
    @select-version="onSelect"
    :icon-type="plugin"
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
