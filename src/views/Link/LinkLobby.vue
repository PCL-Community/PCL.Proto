<script setup lang="ts">
import Dropdown from '@/components/widget/Dropdown.vue'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PHint from '@/components/widget/PHint.vue'
import PInput from '@/components/widget/PInput.vue'
</script>

<template>
  <PHint severity="info">此大厅仅为UI，无联机功能</PHint>
  <PCard title="加入大厅">
    <p>1. 在下方输入朋友发送给你的大厅编号，单击「加入」</p>
    <p>2. 启动游戏，选择「多人游戏」，在局域网游戏中加入大厅</p>
    <div class="hall-input">
      <PInput placeholder="输入大厅编号" style="flex: 1" />
      <PButton inline>清除</PButton>
      <PButton inline>粘贴</PButton>
      <PButton inline type="tint">加入</PButton>
    </div>
  </PCard>
  <PCard title="创建大厅">
    <p>1. 进入世界后，在游戏菜单中选择「对局域网开放」</p>
    <p>2. 在下方选择此游戏实例，单击「创建」</p>
    <p>3. 成功创建大厅后，复制大厅编号并发送给你的朋友</p>
    <div class="hall-input">
      <Dropdown
        :options="[
          {
            key: '1',
            text: '1',
          },
        ]"
        style="flex: 1"
      />
      <PButton inline>刷新</PButton>
      <PButton inline type="tint" disabled>创建</PButton>
    </div>
  </PCard>
</template>

<style lang="css" scoped>
.hall-input {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
</style>
