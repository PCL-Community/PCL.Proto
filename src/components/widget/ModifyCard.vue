<script setup lang="ts">
import PCard from '@/components/widget/PCard.vue'
import { pluginShowText, showIconPath, type pluginType } from '@/util/gameInfo'
import CardInfoItem from './CardInfoItem.vue'
import { ref } from 'vue'
import { onMounted } from 'vue'
import { useTemplateRef } from 'vue'
// import { useSelectedInstance } from '@/stores/gameLaunch'
// import { computed } from 'vue'
const emit = defineEmits<{
  selectVersion: [type: pluginType, versionId: string]
}>()
const cardRef = useTemplateRef('cardRef')
const props = defineProps<{
  plugin: pluginType
  versions?: string[]
  notCompatibleWith?: pluginType
  isLoading: boolean
}>()

function select(versionId: string) {
  emit('selectVersion', props.plugin, versionId)
  selectedVersion.value = versionId
  cardRef.value?.SwitchFoldState()
}

const selectedVersion = ref<string>()
onMounted(() => {
  if (props.plugin == 'vanilla' && props.versions) {
    selectedVersion.value = props.versions[0]
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
      <div v-else-if="selectedVersion" class="version">
        <img :src="showIconPath[plugin]" />
        <span>{{ selectedVersion }} </span>
      </div>
      <div v-else-if="versions" class="sub">
        <span>选择版本</span>
      </div>
    </template>
    <template #content v-if="plugin != 'vanilla' && versions">
      <CardInfoItem
        v-for="version in versions"
        :title="version"
        :click="() => select(version)"
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
