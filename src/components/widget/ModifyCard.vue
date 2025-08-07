<script setup lang="ts">
import PCard from '@/components/widget/PCard.vue'
import { pluginShowText, showIconPath, type pluginType } from '@/util/gameInfo'
import { selectedInstance } from '@/util/gameLaunch'
import { computed } from 'vue'

const props = defineProps<{ type: pluginType }>()

const description = computed(() => {
  return {
    compability:
      props.type == 'vanilla' ||
      selectedInstance.plugins.includes(props.type) ||
      (selectedInstance.plugins.includes('fabric') && props.type == 'fabric-api'),
    version:
      props.type == 'vanilla'
        ? selectedInstance.version
        : selectedInstance.pluginsVersion[props.type],
  }
})
</script>

<template>
  <PCard :default-fold-status="type == 'vanilla' ? 'unfoldable' : 'fold'">
    <template #title> {{ pluginShowText[type] }}</template>
    <template #description>
      <div v-if="description.compability" class="version">
        <img :src="showIconPath[type]" />
        <span>{{ description.version }} </span>
      </div>
      <div v-else class="incompatible">
        <span>与 {{ pluginShowText[selectedInstance.plugins[0]] }} 不兼容</span>
      </div>
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

.incompatible > span {
  color: var(--color-text-grey);
}
img {
  width: 18px;
  height: 18px;
}
</style>
