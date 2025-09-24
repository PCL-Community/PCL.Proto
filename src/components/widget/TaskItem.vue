<script setup lang="ts">
import { TaskStatus, type ITaskItem } from '@/stores/task'
import IconTick from '@/assets/icons/Tick.svg'
import IconPending from '@/assets/icons/Pending.svg'
import TitleClose from '@/assets/icons/TitleClose.svg'

defineProps<ITaskItem>()
</script>

<template>
  <div class="task-item">
    <span class="sub-indicator">
      <span v-if="status === TaskStatus.Pending"><IconPending /></span>
      <span v-else-if="status === TaskStatus.Running">
        {{ (progress * 100).toFixed(0) + '%' }}
      </span>
      <span v-else-if="status === TaskStatus.Completed"><IconTick /></span>
      <span v-else-if="status === TaskStatus.Failed" class="error"><TitleClose /></span>
    </span>
    <span>{{ name }}</span>
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

.sub-indicator > span.error {
  color: var(--color-warn);
}
</style>
