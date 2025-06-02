<script setup lang="ts">
import MyButton from '@/components/widget/MyButton.vue';
import MyCard from '@/components/widget/MyCard.vue';
import { useModal } from '@/composables/useModal';
import { ModalWidthVirant } from '@/types/modal';
// import MinecraftAvatar from '@/components/widget/MinecraftAvatar.vue';
const modal = useModal()

const showDeleteConfirm = async (i: number) => {
    await modal.open({
        title: '模态框标题',
        content: `你点击了第${i}个按钮`,
        width: ModalWidthVirant.Slim,
        buttons: [
            {
                type: 'tint',
                content: '自定义确认',
                operation: () => {
                    console.log('自定义确认逻辑', i)
                }
            },
            {
                type: 'warn',
                content: '自定义取消',
                operation: () => {
                    console.log('自定义取消逻辑', i)
                }
            }
        ]
    })
}
</script>

<template lang="pug">
    MyCard()
    MyCard(hideTitle)
    MyButton(type="tint") 我的按钮
    MyButton(type="warn") 你的按钮
    //- MinecraftAvatar()
    //- MyCard
    MyButton(v-for="i in 8" type="default" @click="showDeleteConfirm(i)") 你的按钮
</template>

<style></style>