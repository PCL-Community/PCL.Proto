<script setup lang="ts">
import PCard from '@/components/widget/PCard.vue'
import { useFloatButton } from '@/composables/useFloatButton'
import { useTaskManager } from '@/stores/task'
import useSideNavState, { defaultWidths } from '@/stores/windowState'
import { onMounted, onUnmounted } from 'vue'
const { floatButtonState } = useFloatButton()
let sideNavState = useSideNavState()
const taskManager = useTaskManager()

onMounted(() => {
  floatButtonState.visible = false
  sideNavState.setWidth(defaultWidths.task_manage)
  taskManager.StartDownloadMCVersion('')
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
        <span class="indicator">{{ taskManager.totalProgress }} %</span>
      </div>
      <div class="task-left-item">
        <span class="header">下载速度</span>
        <div class="divider" />
        <span class="indicator">{{ taskManager.totalSpeed }} B/s</span>
      </div>
      <div class="task-left-item">
        <span class="header">剩余文件</span>
        <div class="divider" />
        <span class="indicator">{{ taskManager.totalRemaining }}</span>
      </div>
    </aside>
    <article class="subview">
      <PCard v-for="task in taskManager.tasks" :key="task.id" :title="task.name">
        <div v-for="item in task.items" :key="item.id">
          <div class="sub-indicator"></div>
          <p>{{ item.name }}</p>
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
