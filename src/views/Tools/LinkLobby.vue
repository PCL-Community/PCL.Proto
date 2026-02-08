<script setup lang="ts">
import Dropdown from '@/components/widget/Dropdown.vue'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PInput from '@/components/widget/PInput.vue'
import sideTip from '@/composables/sideTip'
import useTerracottaStore from '@/stores/terracotta'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const enterLobbyCode = ref<string>()
const terracotta = useTerracottaStore()

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
  } else {
    sideTip.show('房间码无效', 'warn')
  }
}

const createLobby = async (port?: string) => {
  if (!port) return
  try {
    const roomCode = await terracotta.setHostStarting(Number(port), 'PCL.Proto Anonymous Host')
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

onMounted(() => {
  terracotta.update()
  if (terracotta.state == 'host-ok' || terracotta.state == 'guest-ok') {
    router.push({
      path: '/tools/lobby/inner',
      query: {
        code: terracotta.room,
      },
    })
  } else {
    terracotta.startAutoUpdate()
  }
})

onUnmounted(() => {
  terracotta.stopAutoUpdate()
})

const selectedPort = ref<string>()
const customPort = ref<string>()

const dropdownOptions = computed(() => {
  if (!selectedPort.value && terracotta.avaliable_mc_ports.length > 0)
    selectedPort.value = terracotta.avaliable_mc_ports[0]!.toString()
  return terracotta.avaliable_mc_ports.map((port) => ({
    key: port.toString(),
    text: port.toString(),
  }))
})
</script>

<template>
  <PCard :title="$t('tools.link.join_lobby')">
    <p v-for="line in $t('tools.link.join_lobby_description').split('\n')">{{ line }}</p>
    <div class="hall-input">
      <PInput
        :placeholder="$t('tools.link.enter_lobby_code')"
        style="flex: 1"
        v-model="enterLobbyCode"
      />
      <PButton inline @click="enterLobbyCode = ''">清除</PButton>
      <PButton inline>粘贴</PButton>
      <PButton
        inline
        type="tint"
        :click="() => connectWithCode(enterLobbyCode)"
        :disabled="!enterLobbyCode"
        >{{ $t('tools.link.join_lobby') }}</PButton
      >
    </div>
  </PCard>
  <PCard :title="$t('tools.link.create_lobby')">
    <p>1. 进入世界后，在游戏菜单中选择「对局域网开放」</p>
    <p>2. 在下方选择此游戏实例，单击「创建」</p>
    <p>3. 成功创建大厅后，复制大厅编号并发送给你的朋友</p>
    <div class="hall-input">
      <Dropdown :options="dropdownOptions" v-model="selectedPort" style="flex: 1" />
      <PButton inline :click="terracotta.setHostScanning">开始扫描</PButton>
      <PButton inline type="tint" :click="() => createLobby(selectedPort)">创建</PButton>
    </div>
    <div class="hall-input">
      <PInput
        :placeholder="$t('tools.link.enter_custom_port')"
        style="flex: 1"
        v-model="customPort"
      />
      <PButton inline type="tint" :click="() => createLobby(customPort)">指定端口创建</PButton>
    </div>
  </PCard>
  <!-- <PCard title="陶瓦状态"
    ><p style="margin-bottom: 8px;">{{ terracotta.state }}</p>
    <PButton type="tint" :click="terracotta.setWaiting">清除陶瓦状态</PButton></PCard
  > -->
</template>

<style lang="css" scoped>
.hall-input {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-top: 8px;
}
</style>
