<script setup lang="ts">
import GameMemorySet from '@/components/widget/GameMemorySet.vue'
import PCard from '@/components/widget/PCard.vue'
import PCheckbox from '@/components/widget/PCheckbox.vue'
import PHint from '@/components/widget/PHint.vue'
import SetupItem from '@/components/widget/SetupItem.vue'
</script>

<template>
  <PHint severity="info" closable>这些设置只对该游戏实例生效，不影响其他实例。</PHint>
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
      <SetupItem
        label="游戏窗口标题"
        type="input"
        :options="[{ key: 'placeholder', text: '输入游戏窗口标题' }]"
      />
      <PCheckbox label="默认窗口标题" id="default-window-title" />
      <SetupItem label="自定义信息" type="input" />
      <SetupItem label="游戏 Java" type="select" :options="[]" />
    </template>
  </PCard>
  <PCard>
    <template #title>游戏内存</template>
    <template #content>
      <GameMemorySet />
    </template>
  </PCard>
  <PCard>
    <template #title>服务器</template>
    <template #content>
      <SetupItem label="限制验证方式" type="select" :options="[]" />
      <SetupItem label="自动进入服务器" type="input" />
    </template>
  </PCard>
  <PCard default-fold-status="fold">
    <template #title>高级启动选项</template>
    <template #content>
      <SetupItem
        label="JVM 参数头部"
        type="input"
        :options="[{ key: 'placeholder', text: '跟随全局设置' }]"
        title="启动 Minecraft 时使用的额外 JVM 参数，在没有确定把握的情况下请不要尝试修改。支持 Minecraft 实例 JSON 中的字符串替换标记，例如 ${library_directory}。"
      />
      <SetupItem
        label="游戏参数尾部"
        type="input"
        :options="[{ key: 'placeholder', text: '跟随全局设置' }]"
      />
      <SetupItem label="启动前执行命令" type="input" />
      <PCheckbox label="忽略 Java 兼容性警告" id="ignore-java-compatibility-warning" />
      <PCheckbox label="关闭文件校验" id="disable-file-validation" />
      <PCheckbox label="禁用 Java Launch Wrapper" id="disable-java-launch-wrapper" />
      <PCheckbox label="使用代理" id="use-proxy" />
      <PCheckbox label="禁用 RetroWrapper" id="disable-retrowrapper" />
    </template>
  </PCard>
</template>
