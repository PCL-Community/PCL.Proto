<script setup lang="ts">
import CardInfoItem from '@/components/widget/CardInfoItem.vue'
import PCard from '@/components/widget/PCard.vue'
import router from '@/router'
import { PlayerType, TerracottaState, useScaffolding } from '@/stores/scaffolding'
import { invoke } from '@tauri-apps/api/core'
import { useRouteQuery } from '@vueuse/router'
import { onMounted, watch, type Ref } from 'vue'

const roomCode = useRouteQuery('code') as Ref<string | undefined>
const type = useRouteQuery('type') as Ref<PlayerType | undefined>

const closeLobby = async () => {
  console.info('[scaffolding] closing room code', roomCode.value)
  try {
    await invoke('shutdown_room')
    scaffolding.terracotta_state = TerracottaState.Idle
    router.back()
  } catch (error) {
    console.error('[scaffolding] failed to close room code', roomCode.value, error)
  }
}

const scaffolding = useScaffolding()

onMounted(() => {
  if (type.value) {
    scaffolding.playerType = type.value
  }
  if (roomCode.value) {
    scaffolding.roomCode = roomCode.value
  }
})

watch(
  () => scaffolding.terracotta_state,
  (newState) => {
    if (newState == TerracottaState.Exception) {
      router.back()
    }
  },
)
</script>

<template>
  <div class="layout">
    <div class="left-panel" v-card-drop-children-animate>
      <div class="block-1">
        <PCard hide-title>
          <div class="info-panel">
            <div class="name">UserName</div>
            <div v-if="scaffolding.playerType === PlayerType.Host" class="identity">创建者</div>
            <div v-else-if="scaffolding.playerType === PlayerType.Guest" class="identity">
              加入者
            </div>
          </div>
        </PCard>
      </div>
      <div class="block-2">
        <PCard title="大厅信息">
          <div>已连接</div>
          <div>{{ scaffolding.roomCode }}</div>
        </PCard>
      </div>
      <div class="block-3">
        <PCard title="大厅操作">
          <button>复制大厅编号</button>
          <button @click="closeLobby">关闭大厅</button>
        </PCard>
      </div>
    </div>
    <div class="right-panel" v-card-drop-children-animate>
      <PCard title="大厅成员列表（共1人）">
        <CardInfoItem title="PlayerName" subtitle="[主机] PCL.Proto 0.5.6, EasyTier 2.5.0" />
      </PCard>
    </div>
  </div>
</template>

<style scoped>
/* 主要布局容器 */
.layout {
  display: flex;
  gap: 20px;

  /* 左侧面板 - 占2/5宽度，由内容撑开高度 */
  .left-panel {
    flex: 2;
    display: flex;
    flex-direction: column;
    gap: 15px;

    .info-panel {
      display: flex;
      flex-direction: row;
      gap: 5px;
      > div {
        flex: 1;
      }
    }

    > div {
      display: flex;
      flex-direction: column;
    }
  }

  /* 右侧面板 - 占3/5宽度，高度自动对齐左侧 */
  .right-panel {
    flex: 3;
    display: flex;
    flex-direction: column;
  }

  /* 确保所有PCard组件填满各自的容器 */
  :deep(.mycard-container) {
    flex: 1;
  }

  :deep(.mycard) {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  :deep(.mycard-content) {
    flex: 1;
  }
}
</style>
