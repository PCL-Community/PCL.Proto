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
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

const route = useRoute()
const router = useRouter()
const repos = useRepositoriesStore()
const selectedInstance = useSelectedInstance()
const instances = ref<GameInstance[]>([])
const isLoading = ref(false)

watch(
  () => route.params.repository,
  (newRepoIndex, _) => {
    isLoading.value = true
    repos.getInstancesInRepository(Number(newRepoIndex)).then((ins_got) => {
      instances.value = ins_got
      isLoading.value = false
    })
  },
  { immediate: true },
)

function select_instance(instance: GameInstance) {
  invoke('select_instance', {
    repository_index: Number(route.params.repository),
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
  <div class="loading-page" v-if="isLoading">
    <PLoading />
  </div>
  <PCard title="游戏实例" v-else>
    <CardInfoItem
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
