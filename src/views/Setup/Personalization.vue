<script setup lang="ts">
import Dropdown from '@/components/widget/Dropdown.vue'
import PCard from '@/components/widget/PCard.vue'
</script>

<template>
  <PCard title="基础" default-fold-status="unfold">
    <label>不透明度</label>
    <div id="theme-set">
      <label>主题</label><Dropdown :options="[]"></Dropdown><label>浅色配色</label
      ><Dropdown :options="[]"></Dropdown><label>深色配色</label><Dropdown :options="[]"></Dropdown>
    </div>
    <label>字体</label>
  </PCard>
  <PCard title="主页"> </PCard>
</template>

<style lang="css" scoped>
#theme-set {
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
}
</style>
