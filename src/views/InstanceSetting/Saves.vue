<script setup lang="ts">
import PCard from '@/components/widget/PCard.vue'
import PButton from '@/components/widget/PButton.vue'
import CardInfoItem from '@/components/widget/CardInfoItem.vue'

const saves = [
  {
    name: '世界1',
    createTime: '2024/01/13',
    lastEditTime: '2025/08/07',
  },
  {
    name: '世界2',
    createTime: '2024/01/14',
    lastEditTime: '2025/08/08',
  },
]
</script>

<template>
  <p-card title="快捷操作">
    <div class="button-grid">
      <p-button type="tint">打开存档文件夹</p-button>
      <p-button type="default">粘贴剪贴板文件</p-button>
    </div>
  </p-card>
  <p-card :title="`存档列表 (${saves.length})`">
    <card-info-item
      v-for="save in saves"
      :title="save.name"
      :subtitle="`创建时间：${save.createTime}，最近修改时间：${save.lastEditTime}`"
    />
  </p-card>
</template>
