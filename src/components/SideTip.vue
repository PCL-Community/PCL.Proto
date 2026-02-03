<script setup lang="ts">
import sideTip from '@/composables/sideTip'
const tips = sideTip.tips
</script>

<template lang="pug">
#sidetip-container
    TransitionGroup(name="fade")
        .sidetip-item(
            v-for="tip in tips"
            :key="tip.id"
            :class="tip.colorType"
        ) {{ tip.message }}
</template>

<style lang="css" scoped>
#sidetip-container {
  position: absolute;
  left: -8px;
  bottom: 12%;
  z-index: 2;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}

.sidetip-item {
  color: var(--color-background);
  padding: 2px 8px 2px 16px;
  font-size: 12px;
  border-radius: 0 6px 6px 0;
  transition: all 0.4s ease;
  margin-block: 8px;
  display: table;
}

.sidetip-item.default {
  background-color: var(--slight-transparent-blue);
}

.sidetip-item.warn {
  background-color: var(--warn-red);
}

.sidetip-item.success {
  background-color: var(--success-green);
}

.fade-move,
.fade-enter-active,
.fade-leave-active {
  transition:
    transform 0.4s cubic-bezier(0.4, 2, 0.4, 1),
    opacity 0.4s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateX(-10px);
}
</style>
