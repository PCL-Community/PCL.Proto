<script setup lang="ts">
import Dropdown from '@/components/widget/Dropdown.vue'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PInput from '@/components/widget/PInput.vue'
import sideTip from '@/composables/sideTip'
// import type { NetworkInstanceRunningInfo } from '@/types/easytier'
import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const enterLobbyCode = ref<string>()

const connectWithCode = async (code: string) => {
  let roomCode = await invoke<string>('parse_room_code', { code })
  console.info('[scaffolding] connecting to room code', roomCode)
}

const createLobby = async (port: number) => {
  let roomCode = await invoke<string>('start_host', {
    playerName: 'PCL.Proto Anonymous Host',
    port,
  })
  console.info('[scaffolding] created room code', roomCode)
  sideTip.show(`已创建大厅：${roomCode}`, 'success')
  router.push({
    path: '/tools/lobby/inner',
    query: {
      code: roomCode,
    },
  })
}
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
        :click="() => connectWithCode(enterLobbyCode!)"
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
      <PButton inline type="tint" :click="() => createLobby(25565)">创建</PButton>
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
