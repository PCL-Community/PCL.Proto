<script setup lang="ts">
import PCard from '@/components/widget/PCard.vue'
import PInput from '@/components/widget/PInput.vue'
import { pluginShowText, showIconPath, type pluginType } from '@/util/gameInfo'
import { info } from '@tauri-apps/plugin-log'
import { onMounted, onUnmounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ArrowLeft from '@/assets/icons/ArrowLeft.svg'
import ModifyCard from '@/components/widget/ModifyCard.vue'
import { FloatButtonType, useFloatButton } from '@/composables/useFloatButton'
import { listen } from '@tauri-apps/api/event'

const router = useRouter()
const version_id = useRoute().query.version as string
const instance_name = ref(version_id)
const pluginTypes = Object.keys(pluginShowText) as pluginType[]
const { floatButtonState, setFloatButton } = useFloatButton()
var unlistenButton: any

const downloadGame = async () => {
  info(`download game: ${version_id}`)
}

const arrowLeftClicked = () => {
  router.back()
}

onMounted(async () => {
  floatButtonState.visible = true
  setFloatButton(FloatButtonType.DownloadGame)
  unlistenButton = await listen('float-button-click', (event) => {
    if (event.payload === FloatButtonType.DownloadGame) {
      downloadGame().then(() => {
        setFloatButton(FloatButtonType.TaskManage)
        router.back()
      })
    }
  })
})

onUnmounted(() => {
  if (floatButtonState.type === FloatButtonType.DownloadGame) {
    floatButtonState.visible = false
  }
  unlistenButton?.()
})
</script>

<template>
  <PCard hide-title>
    <i class="arrow-left button-animated" @click="arrowLeftClicked"> <ArrowLeft /></i>
    <img :src="showIconPath['vanilla']" />
    <PInput v-model="instance_name" />
  </PCard>
  <ModifyCard v-for="type in pluginTypes" :key="type" :type />
</template>

<style lang="css" scoped>
.arrow-left {
  margin-inline: 6px;
  color: var(--vt-c-gray);
}

img {
  width: 30px;
  height: 30px;
}

:deep(section.mycard-content) {
  flex-direction: row;
  align-items: center;
  gap: 10px;
}

:deep(.input-wrapper) {
  flex: 1;
}
</style>
