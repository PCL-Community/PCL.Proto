<script setup lang="ts">
import CardInfoItem from '@/components/widget/CardInfoItem.vue'
import PCard from '@/components/widget/PCard.vue'
import router from '@/router'
import useTerracottaStore from '@/stores/terracotta'
import { useRouteQuery } from '@vueuse/router'
import IconBtnConnectUserType from '@/assets/icons/IconBtnConnectUserType.svg'
import IconBtnConnectUserName from '@/assets/icons/IconBtnConnectUserName.svg'
import IconBtnFinishQuality from '@/assets/icons/BtnFinishQuality.svg'
import IconBtnFinishId from '@/assets/icons/BtnFinishId.svg'
import { onMounted, onUnmounted } from 'vue'
import sideTip from '@/composables/sideTip'
import { useI18n } from 'vue-i18n'

const terracotta = useTerracottaStore()
const roomCode = useRouteQuery('code')
const t = useI18n()

const closeLobby = async () => {
  terracotta.setWaiting()
  router.back()
  console.info('[scaffolding] closed room code', roomCode.value)
}

const copyRoomCode = () => {
  navigator.clipboard.writeText(roomCode.value as string)
  sideTip.show('复制成功', 'success')
}

onMounted(() => {
  terracotta.update()
  terracotta.startAutoUpdate()
})

onUnmounted(() => {
  terracotta.stopAutoUpdate()
})
</script>

<template>
  <div class="layout">
    <div class="left-panel" v-card-drop-children-animate>
      <div class="block-1">
        <PCard hide-title>
          <div class="info-panel">
            <div class="icon-text">
              <IconBtnConnectUserName width="18" height="18" />
              <span>{{ terracotta.username }}</span>
            </div>
            <div class="icon-text" v-if="terracotta.state == 'host-ok'">
              <IconBtnConnectUserType width="18" height="18" /><span>创建者</span>
            </div>
            <div class="icon-text" v-if="terracotta.state == 'guest-ok'">
              <IconBtnConnectUserType width="16" height="16" /><span>加入者</span>
            </div>
          </div>
        </PCard>
      </div>
      <div class="block-2">
        <PCard title="大厅信息">
          <div class="icon-text lobby-info">
            <IconBtnFinishQuality width="20" height="20" />
            <span>已连接</span>
          </div>
          <div class="icon-text lobby-info">
            <IconBtnFinishId width="18" height="18" />
            <span>{{ roomCode }}</span>
          </div>
        </PCard>
      </div>
      <div class="block-3">
        <PCard title="大厅操作">
          <button @click="copyRoomCode">复制大厅编号</button>
          <button @click="closeLobby">关闭大厅</button>
        </PCard>
      </div>
    </div>
    <div class="right-panel" v-card-drop-children-animate>
      <PCard
        :title="
          `大厅成员列表` +
          (terracotta.profiles?.length ? `（共${terracotta.profiles?.length}人）` : '')
        "
      >
        <CardInfoItem
          v-for="profile in terracotta.profiles"
          :key="profile.machine_id"
          :title="profile.name"
          :subtitle="`[${$t('tools.link.user_type.' + profile.kind)}] ${profile.vendor}`"
        />
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
      flex-wrap: wrap;
      > div {
        flex: 1 1 auto;
        &:nth-child(1) {
          border-right: 1px solid #e5e5e5;
        }
      }
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

.icon-text {
  display: flex;
  flex-direction: row;
  gap: 5px;
  align-items: center;
  justify-content: flex-start;
  &.lobby-info:nth-child(1) {
    margin-bottom: 5px;
  }
  > span {
    pointer-events: all;
    user-select: all;
    -webkit-user-select: all;
  }
}
</style>
