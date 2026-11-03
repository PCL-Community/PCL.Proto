<script setup lang="ts">
import { pclText } from '@/api/pclLocale'
import PCard from '@/components/widget/PCard.vue'
import PHint from '@/components/widget/PHint.vue'
import TaskItem from '@/components/widget/TaskItem.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import { useTaskManager } from '@/stores/task'
const tasks = useTaskManager()
</script>
<template>
  <article class="subview pcl-page speed-right">
    <PCard v-for="task in tasks.tasks" :key="task.id" :title="task.name">
      <template #header-actions>
        <button
          class="pcl-icon-button"
          type="button"
          :disabled="task.cancelling"
          :title="pclText(task.status < 2 ? 'Common.Action.Cancel' : 'Common.Action.Close')"
          :aria-label="pclText(task.status < 2 ? 'Common.Action.Cancel' : 'Common.Action.Close')"
          @click.stop="tasks.cancelTask(task.id)"
        >
          <PSvgIcon name="lucide/x" :size="18" />
        </button>
      </template>
      <div class="task-items">
        <TaskItem v-for="item in task.items" :key="item.id" v-bind="item" :name="item.name" />
        <PHint v-if="task.error" severity="error">{{ task.error }}</PHint>
      </div>
    </PCard>
    <p v-if="!tasks.tasks.length" class="pcl-empty">{{ pclText('Proto.Empty.Tasks') }}</p>
  </article>
</template>
<style scoped>
.speed-right {
  flex: 1;
  min-width: 0;
  overflow-y: auto;
}
.task-items {
  padding: 0 20px 15px;
}
</style>
