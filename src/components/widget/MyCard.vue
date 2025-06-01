<script lang="ts" setup>
defineProps<{ hideTitle?: boolean }>()
</script>

<template lang="pug">
    .mycard
        p(v-if="!hideTitle" class="mycard-title")
            slot(name="title") 标题

        .mycard-content
            slot(name="content") 正文

</template>

<style>
.mycard {
    border-radius: 4px;
    background: var(--color-background-soft);
    box-shadow: 0px 0px 6px rgba(0, 0, 0, 0.1);
    padding: 11px 14px;
    transition: box-shadow 0.4s;
}

.mycard:hover {
    box-shadow: 0px 0px 6px var(--color-tint-shadow);
}

.mycard>.mycard-title {
    font-size: 12px;
    font-weight: bold;
    letter-spacing: 0px;
    line-height: 14.06px;
    text-align: left;
    vertical-align: top;
    transition: color 0.4s;
}

.mycard:hover>.mycard-title {
    color: var(--color-tint);
}

.mycard .mycard-content {
    font-size: 12px;
    margin: 9px;
}
</style>