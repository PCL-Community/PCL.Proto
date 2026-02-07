<script setup lang="ts">
import Dropdown from '@/components/widget/Dropdown.vue'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PInput from '@/components/widget/PInput.vue'
import sideTip from '@/composables/sideTip'
import useTerracottaStore from '@/stores/terracotta'
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const enterLobbyCode = ref<string>()
const terracotta = useTerracottaStore()

const showNotImpledSideTip = () => {
  sideTip.show('当前版本暂不支持此功能', 'warn')
}

const connectWithCode = async (code?: string) => {
  if (!code) return
  terracotta.setGuesting(code, 'PCL.Proto Anonymous Guest')
  console.info('[scaffolding] connecting to room code', code)
}

// 后端尚未实现直接根据端口创建大厅
// const createLobby = async (port: number) => {
//   let roomCode = await invoke<string>('start_host', {
//     playerName: 'PCL.Proto Anonymous Host',
//     port,
//   })
//   console.info('[scaffolding] created room code', roomCode)
//   sideTip.show(`已创建大厅：${roomCode}`, 'success')
//   router.push({
//     path: '/tools/lobby/inner',
//     query: {
//       code: roomCode,
//     },
//   })
// }

const testHostScanning = async () => {
  terracotta.setHostScanning('PCL.Proto Anonymous Host')
}

onMounted(() => {
  terracotta.startAutoUpdate()
})
</script>

<template>
  <PCard :title="$t('link.lobby.join_lobby')">
    <p v-for="line in $t('link.lobby.join_lobby_description').split('\n')">{{ line }}</p>
    <div class="hall-input">
      <PInput
        :placeholder="$t('link.lobby.enter_lobby_code')"
        style="flex: 1"
        v-model="enterLobbyCode"
      />
      <PButton inline>清除</PButton>
      <PButton inline>粘贴</PButton>
      <PButton
        inline
        type="tint"
        :click="() => connectWithCode(enterLobbyCode)"
        :disabled="!enterLobbyCode"
        >{{ $t('link.lobby.join_lobby') }}</PButton
      >
    </div>
  </PCard>
  <PCard :title="$t('link.lobby.create_lobby')">
    <p>1. 进入世界后，在游戏菜单中选择「对局域网开放」</p>
    <p>2. 在下方选择此游戏实例，单击「创建」</p>
    <p>3. 成功创建大厅后，复制大厅编号并发送给你的朋友</p>
    <div class="hall-input">
      <Dropdown :options="[]" style="flex: 1" />
      <PButton inline>刷新</PButton>
      <PButton inline type="tint" :click="showNotImpledSideTip">创建</PButton>
    </div>
  </PCard>
  <PButton inline type="tint" :click="() => testHostScanning()">测试</PButton>
  <PCard title="陶瓦状态">{{ terracotta.state }}</PCard>
</template>

<style lang="css" scoped>
.hall-input {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
</style>
