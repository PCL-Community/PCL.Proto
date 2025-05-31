<script setup lang="ts">
import MyButton from '@/components/widget/MyButton.vue';
import MyCard from '@/components/widget/MyCard.vue';
</script>

<template>
    <MyButton type="tint">我的按钮</MyButton>
    <MyButton type="warn">你的按钮</MyButton>
    <!-- <MyCard /> -->
    <MyButton v-for="i in 12" type="default">你的按钮</MyButton>
</template>

<style></style>