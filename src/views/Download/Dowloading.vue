<script setup lang="ts">
import PCard from '@/components/widget/PCard.vue'
import { useFloatButton } from '@/composables/useFloatButton'
import useSideNavState, { defaultWidths } from '@/stores/windowState'
import { Channel, invoke } from '@tauri-apps/api/core'
import { onMounted, onUnmounted } from 'vue'
const { floatButtonState } = useFloatButton()
let sideNavState = useSideNavState()
const onEvent = new Channel()
let count = 0
onEvent.onmessage = (message) => {
  count += 1
  if (count <= 3) {
    console.log('got task event', message)
  }
}

onMounted(() => {
  floatButtonState.visible = false
  sideNavState.setWidth(defaultWidths.task_manage)
  invoke('download_jars', { on_event: onEvent })
})

onUnmounted(() => {
  floatButtonState.visible = true
})
</script>

<template>
  <div class="view-content">
    <aside>
      <div class="task-left-item">
        <span class="header">总进度</span>
        <div class="divider" />
        <span class="indicator">19.69 %</span>
      </div>
      <div class="task-left-item">
        <span class="header">下载速度</span>
        <div class="divider" />
        <span class="indicator">0 B/s</span>
      </div>
      <div class="task-left-item">
        <span class="header">剩余文件</span>
        <div class="divider" />
        <span class="indicator">0</span>
      </div>
    </aside>
    <article class="subview">
      <PCard title="1.21.8 安装">
        <div>
          <div class="sub-indicator"></div>
          <p>下载原版 Json 文件</p>
        </div>
      </PCard>
    </article>
  </div>
</template>

<style lang="css" scoped>
aside {
  width: v-bind("defaultWidths.task_manage + 'px'");
  flex: 0 0 auto;
  display: flex;
  flex-direction: column;
  justify-content: space-evenly;
  align-items: center;
  padding: 24px;
}

article.subview {
  flex: 1 1 auto;
  overflow-y: auto;
}

.task-left-item {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.task-left-item > .divider {
  width: 100%;
  height: 2px;
  background: var(--color-titlebar);
  margin: 8px 0;
  border: none;
  padding: 0;
}

.task-left-item > .header {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-tint);
}

.task-left-item > .indicator {
  font-size: 18px;
  font-weight: 500;
  color: var(--color-text);
}
</style>
