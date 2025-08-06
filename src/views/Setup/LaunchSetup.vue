<script setup lang="ts">
import SetupItem from '@/components/widget/SetupItem.vue'
import PCard from '@/components/widget/PCard.vue'
import { setup, setupOptions } from '@/util/setup'
import GameMemorySet from '@/components/widget/GameMemorySet.vue'
</script>

<template lang="pug">
  PCard(default-fold-status="unfold")
    template(#title) 启动选项
    template(#content)
        SetupItem(
          v-for="(item, key) in setupOptions.launch"
          :key="key"
          :label="item.label"
          :type="item.type"
          :options="item.options"
          v-model="setup.launch[key]"
        )
  
  PCard()
    template(#title) 游戏内存
    template(#content)
      GameMemorySet()
</template>
