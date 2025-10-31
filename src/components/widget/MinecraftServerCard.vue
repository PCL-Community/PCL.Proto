<script lang="tsx" setup>
import { defineComponent, ref } from 'vue'
import PCard from './PCard.vue'
import PInput from './PInput.vue'
import PButton from './PButton.vue'
import type { MCPingResult, ExtraItem } from '@/types/mcPing'
import { invoke } from '@tauri-apps/api/core'
import { error } from '@tauri-apps/plugin-log'
import sideTip from '@/composables/sideTip'

const serverInput = ref<string>()
const mcPingResult = ref<MCPingResult | null>()
const latency = ref<number>()

async function performQuery() {
  try {
    let addrStr = serverInput.value?.trim()
    if (addrStr) {
      let [result, latency_got] = await invoke<[MCPingResult, number]>('server_query', {
        addrStr,
      })
      console.log('server_query', result)
      mcPingResult.value = result
      latency.value = latency_got
    } else {
      mcPingResult.value = null
      sideTip.show('请输入地址后再查询')
    }
  } catch (err) {
    error(`server query: ${err}`)
    mcPingResult.value = null
    sideTip.show(`${err}`, 'warn')
  }
}

// render description recursively
const DescriptionExtra = defineComponent({
  name: 'DescriptionExtra',
  props: {
    description: {
      type: Object as () => ExtraItem,
      required: true,
    },
  },
  setup(props) {
    return () =>
      typeof props.description === 'string' ? (
        <span style={{ fontSize: '12px' }}>{props.description}</span>
      ) : (
        <>
          {props.description.extra?.map((item) => (
            <DescriptionExtra description={item} />
          ))}
          <span
            style={{
              color: props.description.color,
              fontWeight: props.description.bold ? 'bold' : 'unset',
              fontSize: '12px',
            }}
          >
            {props.description.text}
          </span>
        </>
      )
  },
})
</script>

<template>
  <PCard title="瞅眼服务器">
    <div
      style="
        display: flex;
        gap: 16px;
        align-items: center;
        justify-content: space-around;
        margin-bottom: 8px;
      "
    >
      <PInput v-model="serverInput" placeholder="输入服务器地址" style="flex: 1" />
      <PButton inline :click="performQuery" :disabled="!serverInput">查询</PButton>
    </div>
    <div class="server-card" v-if="mcPingResult">
      <img class="server-favicon" v-if="mcPingResult.favicon" :src="mcPingResult.favicon" />
      <div class="server-title">
        <p style="font-size: 15px; font-weight: bold">
          服务器版本: {{ mcPingResult.version.name }}
        </p>
        <!-- 描述部分需要递归渲染 -->
        <DescriptionExtra :description="mcPingResult.description" />
      </div>
      <div class="server-info">
        <p>{{ mcPingResult.players.online }}/{{ mcPingResult.players.max }}</p>
        <p class="latency-text">{{ latency }}ms</p>
      </div>
    </div>
  </PCard>
</template>

<style scoped>
.server-card {
  background-image: url('@/assets/pictures/server_bg.png');
  background-size: auto;
  background-repeat: repeat-x;
  image-rendering: pixelated;
  height: 60px;
  border-radius: 4px;
  padding: 6px;
  color: white;
}

.server-favicon {
  image-rendering: pixelated;
  height: 100%;
  float: left;
}

.server-title {
  float: left;
  margin-left: 10px;
  max-width: 80%;
}

.server-info {
  float: right;
}

.latency-text {
  color: rgba(155, 240, 11, 1);
}
</style>
