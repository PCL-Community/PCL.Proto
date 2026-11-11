<script lang="tsx" setup>
import { pclText } from '@/api/pclLocale'
import { pclImagePath } from '@/api/pclAssets'
import { defineComponent, ref } from 'vue'
import PCard from './PCard.vue'
import PTextBox from './PTextBox.vue'
import PButton from './PButton.vue'
import type { MCPingResult, ExtraItem } from '@/types/mcPing'
import { isTauri, invoke } from '@tauri-apps/api/core'
import { error } from '@tauri-apps/plugin-log'
import sideTip from '@/composables/sideTip'
import { renderMinecraftText, minecraftTextColor, minecraftTextStyle } from '@/util/minecraftText'

defineProps<{ embedded?: boolean }>()
const serverInput = ref<string>()
const mcPingResult = ref<MCPingResult | null>()
const latency = ref<number>()
/// 背景沿用 CE 的资源目录，不再额外打包一份副本。
const serverBackground = `url("${pclImagePath(
  'images/Backgrounds/server_bg.png',
  import.meta.env.BASE_URL,
)}")`

async function performQuery() {
  try {
    let addrStr = serverInput.value?.trim()
    if (addrStr) {
      let [result, latency_got] = await invoke<[MCPingResult, number]>('server_query', {
        addrStr,
      })
      mcPingResult.value = result
      latency.value = latency_got
    } else {
      mcPingResult.value = null
      sideTip.show(pclText('Proto.Server.EnterAddress'))
    }
  } catch (err) {
    if (isTauri()) void error(`server query: ${err}`)
    mcPingResult.value = null
    sideTip.show(`${err}`, 'warn')
  }
}

// render description recursively
const DescriptionExtra = defineComponent(
  (props: { description: ExtraItem }) => {
    return () =>
      typeof props.description === 'string' ? (
        <span>
          {renderMinecraftText(props.description).map((run, index) => (
            <span key={index} style={minecraftTextStyle(run)}>
              {run.text}
            </span>
          ))}
        </span>
      ) : (
        <>
          {props.description.extra?.map((item) => (
            <DescriptionExtra description={item} />
          ))}
          {props.description.text &&
            renderMinecraftText(props.description.text, {
              color: minecraftTextColor(props.description.color),
              bold: props.description.bold,
            }).map((run, index) => (
              <span key={index} style={minecraftTextStyle(run)}>
                {run.text}
              </span>
            ))}
        </>
      )
  },
  { name: 'DescriptionExtra', props: ['description'] },
)
</script>

<template>
  <component :is="embedded ? 'div' : PCard" :title="pclText('Tools.Test.ServerQuery.Title')">
    <div
      style="
        display: flex;
        gap: 16px;
        align-items: center;
        justify-content: space-around;
        margin-bottom: 8px;
      "
    >
      <PTextBox
        v-model="serverInput"
        :placeholder="pclText('Tools.ServerQuery.Address.Hint')"
        style="flex: 1"
      />
      <PButton inline @click="performQuery" :disabled="!serverInput">{{
        pclText('Tools.ServerQuery.Action.Query')
      }}</PButton>
    </div>
    <div class="server-card" v-if="mcPingResult">
      <div>
        <img class="server-favicon" v-if="mcPingResult.favicon" :src="mcPingResult.favicon" />
        <div class="server-title">
          <p style="font-size: 15px; font-weight: bold">
            {{ pclText('Proto.Server.Version', [mcPingResult.version.name]) }}
          </p>
          <!-- 描述部分需要递归渲染 -->
          <p>
            <DescriptionExtra :description="mcPingResult.description" />
          </p>
        </div>
      </div>
      <div class="server-info">
        <p>{{ mcPingResult.players.online }}/{{ mcPingResult.players.max }}</p>
        <p class="latency-text">{{ latency }}ms</p>
      </div>
    </div>
  </component>
</template>

<style scoped>
.server-card {
  background-image: v-bind(serverBackground);
  background-size: auto;
  background-repeat: repeat-x;
  image-rendering: pixelated;
  border-radius: 4px;
  padding: 6px;
  color: white;
  display: flex;
  flex-direction: row;
  align-items: start;
  justify-content: space-between;
  gap: 10px;
}

.server-favicon {
  image-rendering: pixelated;
  float: left;
  margin-right: 10px;
}

.server-info {
  float: right;
}

.latency-text {
  color: rgba(155, 240, 11, 1);
}
</style>
