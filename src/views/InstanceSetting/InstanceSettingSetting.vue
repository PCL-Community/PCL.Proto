<script setup lang="ts">
import PCard from '@/components/widget/PCard.vue'
import PHint from '@/components/widget/PHint.vue'
import SetupItem from '@/components/widget/SetupItem.vue'
</script>

<template>
  <PHint severity="info">这些设置只对该游戏实例生效，不影响其他实例。</PHint>
  <PCard>
    <template #title>启动选项</template>
    <template #content>
      <SetupItem
        label="版本隔离"
        type="select"
        :options="[
          { key: 'on', text: '开启' },
          { key: 'off', text: '关闭' },
        ]"
      ></SetupItem>
    </template>
  </PCard>
  <PCard>
    <template #title>游戏内存</template>
    <template #content> </template>
  </PCard>
</template>
