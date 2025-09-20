<script setup lang="ts">
import { TaskStatus, type ITaskItem } from '@/stores/task'
import IconTick from '@/assets/icons/Tick.svg'
import IconPending from '@/assets/icons/Pending.svg'

const props = defineProps<ITaskItem>()
</script>

<template>
  <div class="task-item">
    <span class="sub-indicator">
      <span v-if="props.status === TaskStatus.Pending"><IconPending /></span>
      <span v-else-if="props.status === TaskStatus.Running">
        {{ (props.progress * 100).toFixed(0) + '%' }}
      </span>
      <span v-else-if="props.status === TaskStatus.Completed"><IconTick /></span>
    </span>
    <span>{{ props.name }}</span>
  </div>
</template>

<style scoped>
.task-item {
  display: inline-flex;
  align-items: center;
  justify-content: flex-start;
  gap: 10px;
}

.sub-indicator {
  width: 30px;
  display: block;
  color: var(--color-tint);
}

.sub-indicator > span {
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
