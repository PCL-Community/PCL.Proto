<script lang="ts" setup>
import { ref } from 'vue'
import PCard from './PCard.vue'
import PInput from './PInput.vue'
import PButton from './PButton.vue'
import type { MCPingResult } from '@/types/mcPing'
import { invoke } from '@tauri-apps/api/core'
import { error } from '@tauri-apps/plugin-log'

const serverInput = ref<string>()
const cardVisible = ref<boolean>(false)
const mcPingResult = ref<MCPingResult>()
const latency = ref<number>()

async function performQuery() {
  try {
    let [result, latency_got] = await invoke<[MCPingResult, number]>('server_query', {
      addrStr: serverInput.value?.trim(),
    })
    console.log('server_query', result)
    mcPingResult.value = result
    latency.value = latency_got
    cardVisible.value = true
  } catch (err) {
    error(`server query: ${err}`)
    cardVisible.value = false
  }
}
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
      <PButton inline :click="performQuery">查询</PButton>
    </div>
    <div class="server-card" v-if="cardVisible">
      <img
        class="server-favicon"
        :src="
          mcPingResult?.favicon ??
          'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+P+/HgAFhAJ/wlseKgAAAABJRU5ErkJggg=='
        "
      />
      <div class="server-title">
        <p style="font-size: 15px; font-weight: bold">Minecraft 服务器</p>
        <p style="font-size: 12px">{{ mcPingResult?.description }}</p>
      </div>
      <div class="server-info">
        <p>{{ mcPingResult?.players.online }}/{{ mcPingResult?.players.max }}</p>
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
}

.server-info {
  float: right;
}

.latency-text {
  color: rgba(155, 240, 11, 1);
}
</style>
