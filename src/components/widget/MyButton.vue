<!-- 一个自定义的按钮：传入参数tint，若为true则为高亮 -->

<script setup lang="ts">
type ButtonType = 'default' | 'tint' | 'warn'
defineProps<{ type?: ButtonType }>()

</script>

<template lang="pug">
    button.mybutton(:class="type ?? 'default'")
        slot 我的按钮
</template>

<style scoped>
* {
    font-size: 13px;
}

button.mybutton {
    padding: 0.7rem;
    background-color: transparent;
    border-radius: 4px;
    border: 1px solid rgba(52, 61, 74, 1);
    transition: all 0.2s;
}

button.tint {
    color: var(--color-tint);
    border-color: var(--color-tint);
}

button.warn {
    color: var(--color-warn);
    border-color: var(--color-warn);
}

button.mybutton:hover {
    background-color: var(--color-tint-lighter);
    color: var(--color-tint);
    border-color: var(--color-tint);
}

button.warn:hover {
    background-color: var(--color-warn-lighter);
    color: var(--color-warn);
    border-color: var(--color-warn);
}

button.mybutton:active {
    transform: scale(0.95);
}
</style>