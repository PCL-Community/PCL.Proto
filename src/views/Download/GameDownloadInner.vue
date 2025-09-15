<script setup lang="ts">
import PCard from '@/components/widget/PCard.vue'
import PInput from '@/components/widget/PInput.vue'
import { pluginShowText, showIconPath, type pluginType } from '@/util/gameInfo'
import { info } from '@tauri-apps/plugin-log'
import { ref } from 'vue'
import { useRoute } from 'vue-router'
import ArrowLeft from '@/assets/icons/ArrowLeft.svg'
import ModifyCard from '@/components/widget/ModifyCard.vue'
import FloatButton from '@/components/widget/FloatButton.vue'
import IconDownload from '@/components/icons/header/IconDownload.vue'

const version_id = useRoute().query.version as string
const instance_name = ref(version_id)
info(`ready to download game: ${version_id}`)
const pluginTypes = Object.keys(pluginShowText) as pluginType[]

const downloadGame = () => {
  info(`download game: ${version_id}`)
}
</script>

<template>
  <PCard hide-title>
    <ArrowLeft class="arrow-left" />
    <img :src="showIconPath['vanilla']" />
    <PInput v-model="instance_name" />
  </PCard>
  <ModifyCard v-for="type in pluginTypes" :key="type" :type />
  <!-- TODO: 按钮放在全局，切换到此页面时显示出来，以便于后面做切换 -->
  <FloatButton @click="downloadGame" text="开始下载" :icon="IconDownload" />
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
