<script setup lang="ts">
import PCard from '@/components/widget/PCard.vue'
import { pluginShowText, showIconPath, type pluginType } from '@/util/gameInfo'
import CardInfoItem from './CardInfoItem.vue'
import { ref } from 'vue'
import { onMounted } from 'vue'
import { useTemplateRef } from 'vue'
import type { McPluginReport } from '@/types/mcPlugin'
// import { useSelectedInstance } from '@/stores/gameLaunch'
// import { computed } from 'vue'
const emit = defineEmits<{
  selectVersion: [type: pluginType, versionId: string]
}>()
const cardRef = useTemplateRef('cardRef')
const props = defineProps<{
  plugin: pluginType
  versions?: McPluginReport[]
  notCompatibleWith?: pluginType
  isLoading: boolean
}>()

function select(versionId: string) {
  emit('selectVersion', props.plugin, versionId)
  selectedVersionId.value = versionId
  cardRef.value?.SwitchFoldState()
}

const selectedVersionId = ref<string>()
onMounted(() => {
  if (props.plugin == 'vanilla' && props.versions) {
    selectedVersionId.value = props.versions[0]?.version
  }
})
</script>

<template>
  <PCard :default-fold-status="plugin == 'vanilla' ? 'unfoldable' : 'fold'" ref="cardRef">
    <template #title> {{ pluginShowText[plugin] }}</template>
    <template #description>
      <div v-if="notCompatibleWith" class="sub">
        <span>与 {{ notCompatibleWith }} 不兼容</span>
      </div>
      <div v-else-if="isLoading" class="sub">
        <span>加载中...</span>
      </div>
      <div v-else-if="selectedVersionId" class="version">
        <img :src="showIconPath[plugin]" />
        <span>{{ selectedVersionId }} </span>
      </div>
      <div v-else-if="versions" class="sub">
        <span>选择版本</span>
      </div>
    </template>
    <template #content v-if="plugin != 'vanilla' && versions">
      <CardInfoItem
        v-for="version in versions"
        :title="version.version"
        :subtitle="version.stable == null ? undefined : version.stable ? '稳定版' : '开发版'"
        :click="() => select(version.version)"
        :icon="showIconPath[plugin]"
      />
    </template>
  </PCard>
</template>

<style scoped>
.version {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 6px;
}

span {
  text-align: left;
  font-size: 12px;
}

.sub > span {
  color: var(--color-text-grey);
}
img {
  width: 18px;
  height: 18px;
}
</style>
