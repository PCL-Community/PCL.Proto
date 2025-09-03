<script setup lang="ts">
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import SetupItem from '@/components/widget/SetupItem.vue'
import { ref } from 'vue'
import OverviewCard from './OverviewCard'

const icon = ref<string>('auto')
const category = ref<string>('auto')
</script>

<template>
  <OverviewCard />
  <PCard>
    <template #title>个性化</template>
    <template #content>
      <SetupItem
        label="图标"
        type="select"
        :options="[
          { key: 'auto', text: '自动' },
          { key: 'custom', text: '自定义' },
          { key: 'cubblestone', text: '圆石' },
          { key: 'command', text: '命令方块' },
        ]"
        v-model:model-value="icon"
      />
      <SetupItem
        label="分类"
        type="select"
        :options="[
          { key: 'auto', text: '自动' },
          { key: 'hidden', text: '从实例列表中隐藏' },
          { key: 'regular', text: '常规实例' },
          { key: 'uncommon', text: '不常用实例' },
          { key: 'foolday', text: '愚人节版本' },
        ]"
        v-model:model-value="category"
      />
      <div class="button-grid">
        <PButton>修改实例名</PButton>
        <PButton>修改实例描述</PButton>
        <PButton>从收藏夹中移除</PButton>
      </div>
    </template>
  </PCard>

  <PCard>
    <template #title>快捷方式</template>
    <template #content>
      <div class="button-grid">
        <PButton>实例文件夹</PButton>
        <PButton>存档文件夹</PButton>
        <PButton>Mod 文件夹</PButton>
      </div>
    </template>
  </PCard>

  <PCard>
    <template #title>高级管理</template>
    <template #content>
      <div class="button-grid">
        <PButton>导出启动脚本</PButton>
        <PButton>测试游戏</PButton>
        <PButton>补全文件</PButton>
        <PButton>重置</PButton>
        <PButton>查看启动次数</PButton>
        <PButton type="warn">删除实例</PButton>
      </div>
    </template>
  </PCard>
</template>
