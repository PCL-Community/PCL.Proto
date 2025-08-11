<script setup lang="ts">
import SetupItem from '@/components/widget/SetupItem.vue'
import PCard from '@/components/widget/PCard.vue'
import { useSetup, type ISetupOption, type SetupItemType } from '@/stores/setup'
import GameMemorySet from '@/components/widget/GameMemorySet.vue'
import { PStorageKeys } from '@/stores/localStorage'

const setupOptions = {
  launch: {
    defaultVersionIsolation: {
      type: 'select' as SetupItemType,
      label: '默认版本隔离',
      options: [
        { key: 'disabled', text: '关闭' },
        { key: 'isolate-moddable', text: '隔离可安装 Mod 的版本' },
        { key: 'isolate-nonformal', text: '隔离非正式版' },
        { key: 'isolate-moddable-and-nonformal', text: '隔离可安装 Mod 的版本与非正式版本' },
        { key: 'isolate-all', text: '隔离所有版本' },
      ] as ISetupOption[],
    },
    gameWindowTitle: {
      type: 'input' as SetupItemType,
      options: [{ key: 'placeholder', text: '默认' }],
      label: '游戏窗口标题',
    },
    customInfo: {
      type: 'input' as SetupItemType,
      options: [{ key: 'placeholder', text: '默认' }],
      label: '自定义信息',
    },
    launcherVisibility: {
      type: 'select' as SetupItemType,
      label: '启动器可见性',
      options: [{ key: 'close-immediately', text: '游戏启动后立即关闭' }] as ISetupOption[],
    },
  },
}

const setup = useSetup()

setup.$subscribe((mutation, state) => {
  console.log('[setup] changed:', state)
  localStorage.setItem(PStorageKeys.Setup, JSON.stringify(state))
})
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
