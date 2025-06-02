<script setup lang="ts">
import MyButton from '@/components/widget/MyButton.vue';
import MyCard from '@/components/widget/MyCard.vue';
import { useModal } from '@/composables/useModal';
const modal = useModal()
const showDeleteConfirm = async (i: number) => {
    const confirmed = await modal.open({
        title: '删除确认',
        content: '确定要删除这条记录吗？'
    })
    if (confirmed) {
        console.log("删除了" + i)
    }
}
</script>

<template lang="pug">
    MyButton(type="tint") 我的按钮
    MyButton(type="warn") 你的按钮
    //- MyCard
    MyButton(v-for="i in 12" type="default" @click="showDeleteConfirm(i)") 你的按钮
</template>

<style></style>