<script setup lang="ts">
import sideTip from '@/composables/sideTip';
const tips = sideTip.tips;
</script>

<template lang="pug">
#sidetip-container
    TransitionGroup(name="sidetip" tag="div")
        .sidetip-item(
            v-for="tip in tips"
            :key="tip.id"
            :class="tip.colorType"
        ) {{ tip.message }}
</template>

<style lang="css" scoped>
#sidetip-container {
    position: absolute;
    left: 0;
    bottom: 12%;
    z-index: 2;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
}

.sidetip-item {
    color: var(--color-background);
    padding: 2px 8px;
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

.sidetip-move,
.sidetip-enter-active,
.sidetip-leave-active {
    transition: all 0.4s cubic-bezier(.4, 2, .4, 1);
}

.sidetip-enter-from,
.sidetip-leave-to {
    opacity: 0;
    transform: translateX(-10px);
}
</style>