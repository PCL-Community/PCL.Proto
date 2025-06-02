<script setup lang="ts">
import MyButton from '@/components/widget/MyButton.vue';
import MyCard from '@/components/widget/MyCard.vue';
import { useModal } from '@/composables/useModal';
import { ModalWidthVirant } from '@/types/modal';
const modal = useModal()
const showDeleteConfirm = async (i: number) => {
    const confirmed = await modal.open({
        title: '导入 / 导出档案',
        content: 'PCL Proto 不支持导入 HMCL 的全局账户列表，也不能导出档案列表至 HMCL 全局账户列表。😄',
        width: ModalWidthVirant.Slim
    })
    if (confirmed) {
        console.log("点击了按钮" + i)
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