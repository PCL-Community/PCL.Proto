<script setup lang="ts">
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PCheckbox from '@/components/widget/PCheckbox.vue'
import PHint from '@/components/widget/PHint.vue'
import SetupItem from '@/components/widget/SetupItem.vue'
import { useSelectedInstance } from '@/stores/gameLaunch'
const selectedInstance = useSelectedInstance()
</script>

<template>
  <p-card hide-title>
    <template #content>
      <div style="display: flex; gap: 2rem">
        <setup-item
          type="input"
          label="整合包名称"
          style="flex: 3"
          :options="[{ key: 'placeholder', text: selectedInstance.name }]"
        />
        <setup-item
          type="input"
          label="整合包版本"
          style="flex: 2"
          :ratio="2"
          :options="[{ key: 'placeholder', text: '1.0.0' }]"
        />
      </div>
    </template>
  </p-card>
  <p-card>
    <template #title>导出内容列表 </template>
    <template #content>
      <p-checkbox label="游戏本体" id="游戏本体" />
      <p-checkbox label="Mod" id="Mod" />
      <p-checkbox label="光影包" id="光影包" />
      <p-checkbox label="单人游戏存档" id="单人游戏存档" />
      <p-checkbox label="多人服务器列表" id="多人服务器列表" />
      <p-checkbox label="PCL启动器程序" id="PCL启动器程序" />
    </template>
  </p-card>
  <p-card>
    <template #title>高级选项</template>
    <template #content>
      <p-checkbox label="打包资源文件，以避免在导出时下载" id="pack-resource" />
      <p-checkbox label="Modrinth 上传模式" id="modrinth-upload" />
      <p-hint severity="info" closable
        >配置文件中含有更多高级选项，例如精准控制导出的文件、设置整合包存放位置等。<br />
        要修改这些选项，请先点击“保存配置”，在编辑配置文件后再导入。</p-hint
      >
      <div class="button-grid">
        <p-button type="tint">读取配置</p-button>
        <p-button>保存配置</p-button>
      </div>
    </template>
  </p-card>
</template>
