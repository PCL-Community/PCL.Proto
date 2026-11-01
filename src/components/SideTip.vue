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
  color: white;
  padding: 2px 8px 2px 16px;
  font-size: 12px;
  border-radius: 0 6px 6px 0;
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

.fade-move {
  transition: transform 400ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.fade-enter-active {
  /* CE MyToast：进入位移 400 ms、透明 100 ms。 */
  transition:
    transform 400ms cubic-bezier(0.33, 1, 0.68, 1),
    opacity 100ms linear;
}

.fade-leave-active {
  /* CE MyToast：退出位移 200 ms、透明 150 ms。 */
  transition:
    transform 200ms cubic-bezier(0.55, 0, 1, 0.45),
    opacity 150ms linear;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateX(-10px);
}
</style>
