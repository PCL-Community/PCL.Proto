<script setup lang="ts">
import Dropdown from '@/components/widget/Dropdown.vue'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PInput from '@/components/widget/PInput.vue'
import sideTip from '@/composables/sideTip'
import useTerracottaStore from '@/stores/terracotta'
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const enterLobbyCode = ref<string>()
const terracotta = useTerracottaStore()

const showNotImpledSideTip = () => {
  sideTip.show('当前版本暂不支持此功能', 'warn')
}

const connectWithCode = async (code?: string) => {
  if (!code) return
  const success = await terracotta.setGuesting(code, 'PCL.Proto Anonymous Guest')
  if (success) {
    console.info('[scaffolding] connecting to room code', code)
    router.push({
      path: '/tools/lobby/inner',
      query: {
        code,
      },
    })
  }
}

const createLobbyWithSelectedPort = async () => {
  if (!selectedPort.value) return
  try {
    const roomCode = await terracotta.setHostStarting(Number(selectedPort.value), 'PCL.Proto Anonymous Host')
    sideTip.show(`已创建大厅：${roomCode}`, 'success')
    router.push({
      path: '/tools/lobby/inner',
      query: {
        code: roomCode,
      },
    })
  } catch (err) {
    console.error('[terracotta] create lobby failed', err)
    sideTip.show('创建大厅失败', 'warn')
    return
  }
}

const customPortInput = () => {}

onMounted(() => {
  terracotta.startAutoUpdate()
})

const selectedPort = ref<string>()
const dropdownOptions = computed(() =>
  terracotta.avaliable_mc_ports.map((port) => ({ key: port.toString(), text: port.toString() })),
)
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
      <Dropdown :options="dropdownOptions" v-model="selectedPort" style="flex: 1" />
      <PButton inline :click="terracotta.setHostScanning">开始扫描</PButton>
      <PButton inline type="tint" :click="createLobbyWithSelectedPort">创建</PButton>
      <PButton inline :click="customPortInput">指定端口</PButton>
    </div>
  </PCard>
  <PCard title="陶瓦状态"
    >{{ terracotta.state }}
    <PButton type="tint" :click="terracotta.setWaiting">清除陶瓦状态</PButton></PCard
  >
</template>

<style lang="css" scoped>
.hall-input {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
</style>
