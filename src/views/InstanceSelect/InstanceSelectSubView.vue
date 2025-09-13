<script setup lang="ts">
import CardInfoItem from '@/components/widget/CardInfoItem.vue'
import PCard from '@/components/widget/PCard.vue'
import PLoading from '@/components/widget/PLoading.vue'
import { useRepositoriesStore } from '@/stores/repositories'
import type GameInstance from '@/types/gameInstance'
import { showIconPath } from '@/util/gameInfo'
import { ref, watch } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const repos = useRepositoriesStore()
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
    ></CardInfoItem>
  </PCard>
  <PCard>
    <template #title> 收藏夹 </template>
  </PCard>
</template>
