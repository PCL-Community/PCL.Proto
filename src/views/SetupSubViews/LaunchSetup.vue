<script setup lang="ts">
import MyCard from '@/components/widget/MyCard.vue';

</script>

<template lang="pug">
    MyCard()
        template(#title) 启动选项
        template(#content)
            p 该页面用于设置启动选项。
</template>