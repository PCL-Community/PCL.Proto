<script setup lang="ts">
import CardInfoItem from '@/components/widget/CardInfoItem.vue'
import PCard from '@/components/widget/PCard.vue'
import PLoading from '@/components/widget/PLoading.vue'
import sideTip from '@/composables/sideTip'
import { useSelectedInstance } from '@/stores/gameLaunch'
import { useRepositoriesStore } from '@/stores/repositories'
import type GameInstance from '@/types/gameInstance'
import { showIconPath } from '@/util/gameInfo'
import { invoke } from '@tauri-apps/api/core'
import { error, info } from '@tauri-apps/plugin-log'
import { useRouteParams } from '@vueuse/router'
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'

const repository = useRouteParams('repository', 0, { transform: Number })
const router = useRouter()
const reposStore = useRepositoriesStore()
const selectedInstance = useSelectedInstance()
const instances = ref<GameInstance[]>([])
const isLoading = ref(false)

watch(
  repository,
  (newRepoIndex, _) => {
    isLoading.value = true
    reposStore.getInstancesInRepository(newRepoIndex).then((ins_got) => {
      instances.value = ins_got
      isLoading.value = false
    })
  },
  { immediate: true },
)

function select_instance(instance: GameInstance) {
  invoke('select_instance', {
    repository_index: repository.value,
    instance_id: instance.id,
  })
    .then(() => {
      selectedInstance.$patch({ instance_info: instance })
      info(`select_instance success: ${instance.id}`)
      sideTip.show(`Successfully selected ${instance.id}`, 'success')
      console.log(selectedInstance.plugins)
      router.push('/home')
    })
    .catch((err) => {
      error(`select_instance error: ${err}`)
    })
}
</script>

<template>
  <PCard title="游戏实例" ref="instanceCard">
    <PLoading v-if="isLoading" :card="false" />
    <CardInfoItem
      v-else
      v-for="item in instances"
      :key="item.id"
      :title="item.name"
      :subtitle="item.version"
      :icon="showIconPath['grass']"
      :click="() => select_instance(item)"
    ></CardInfoItem>
  </PCard>
  <PCard>
    <template #title> 收藏夹 </template>
  </PCard>
</template>
