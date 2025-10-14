<script setup lang="ts">
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import { useModal } from '@/composables/useModal'
import sideTip from '@/composables/sideTip'
import { ModalWidthVirant } from '@/types/modal'
import { ref } from 'vue'
import PLoading, { type LoadingState } from '@/components/widget/PLoading.vue'
import PHint from '@/components/widget/PHint.vue'
// import MinecraftAvatar from '@/components/widget/MinecraftAvatar.vue';
const modal = useModal()
const count = ref<number>(0)
function presentBtnClick() {
  let text: string = '耶✌️'.repeat(++count.value)
  sideTip.show(count.value < 10 ? text : text + '💥')
  setTimeout(() => --count.value, 2000)
}

const showDeleteConfirm = async (i: number) => {
  if (i == 3) {
    let result = await modal.open({ title: '默认模态框行为', content: '默认行为包含文本' })
    console.log('获得默认模态框结果', result)
  } else {
    let result = await modal.open({
      title: '模态框标题',
      content: `你点击了第${i}个按钮`,
      width: ModalWidthVirant.Slim,
      buttons: [
        {
          type: 'tint',
          content: '自定义确认',
          operation: () => {
            console.log('自定义确认逻辑', i)
            modal.close(true)
          },
        },
        {
          type: 'warn',
          content: '自定义取消',
          operation: () => {
            console.log('自定义取消逻辑', i)
            modal.close(false)
          },
        },
      ],
    })
    console.log('自定义模态框返回结果', result)
  }
}

const loadingState = ref<LoadingState>('loading')
</script>

<template lang="pug">
    PCard(defaultFoldStatus="unfold")
        template(#title) 欢迎来到 PCL.Proto！
        template(#content)
            PHint(severity="info") 随着PCL的分支版本竞相启动，UI的还原成为了一大困扰众开发者的难题。#[span.tint PCL.Proto] 应运而生。
            p 本项目以PCL2（龙腾猫跃）和PCL2-CE为蓝本。旨在为各PCL分支版本提供一个标准化的原型样本。该仓库使用 Vue3 搭建，如果你的仓库使用 Webview 作为前端，则可以直接引用该项目。

    PButton(type="tint" :click="presentBtnClick") 点击这个按钮会有提示哦 😬

    PLoading(:state='loadingState' :card="true")

    #loading-control
        PButton(type="tint" :click="() => loadingState = 'loading'") 设置为loading
        PButton(type="warn" :click="() => loadingState = 'error'") 设置为error

    PCard(hideTitle)
        template(#content)
            p 卡片可自由配置，例如，这是一张不可折叠无标题卡片

    PCard(defaultFoldStatus="unfoldable")
        template(#title)
            p 而这是一张不可折叠有标题卡片

    PButton(type="warn" :click="()=>sideTip.show('你的按钮但是文字比较长', 'warn')") 你的按钮
    //- MinecraftAvatar()
    //- PCard
    PButton(v-for="i in 3" type="default" :click="() => showDeleteConfirm(i)") 你的按钮 {{ i }}
</template>

<style lang="css" scoped>
#loading-control {
  display: flex;
  justify-content: space-around;
}
</style>
