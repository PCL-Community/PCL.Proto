<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue'
import SetupItem from '@/components/widget/SetupItem.vue'
import PCard from '@/components/widget/PCard.vue'
import PCheckBox from '@/components/widget/PCheckBox.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PErrorRetry from '@/components/widget/PErrorRetry.vue'
import PHint from '@/components/widget/PHint.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PTextBox from '@/components/widget/PTextBox.vue'
import sideTip from '@/composables/sideTip'
import { useModal } from '@/composables/useModal'
import { useNavAction } from '@/composables/navAction'
import { pclText } from '@/api/pclLocale'
import type { LinkProtocol, LinkSettings } from '@/api/link'
import { useLinkStore, linkErrorMessage } from '@/stores/link'

const link = useLinkStore()
const form = reactive<LinkSettings>({
  username: '',
  protocol: 'tcp',
  latencyFirst: true,
  tryPunchSym: true,
  enableIpv6: true,
  enableCliOutput: false,
})
const loaded = ref(false)
const loadError = ref<string | null>(null)
const saveError = ref<string | null>(null)
let editVersion = 0

const protocolOptions = computed(() => [
  {
    key: 'tcp',
    text: pclText('Setup.GameLink.Protocol.Tcp'),
    title: pclText('Setup.GameLink.Protocol.Tcp.ToolTip'),
  },
  {
    key: 'udp',
    text: pclText('Setup.GameLink.Protocol.Udp'),
    title: pclText('Setup.GameLink.Protocol.Udp.ToolTip'),
  },
])

async function load() {
  loadError.value = null
  try {
    Object.assign(form, await link.loadSettings())
    loaded.value = true
  } catch (error) {
    loadError.value = linkErrorMessage(error, 'Setup.GameLink.Error.InitFailed')
    sideTip.show(loadError.value, 'warn')
  }
}

async function save(patch?: Partial<LinkSettings>) {
  if (!loaded.value || link.busy.resetting) return
  if (patch) Object.assign(form, patch)
  const next: LinkSettings = { ...form }
  const current = ++editVersion
  saveError.value = null
  try {
    await link.saveSettings(next)
  } catch (error) {
    if (current !== editVersion) return
    saveError.value = linkErrorMessage(error, 'Setup.GameLink.Error.ConfigChangeFailed')
    sideTip.show(saveError.value, 'warn')
  }
}

function setProtocol(value?: string) {
  if (!value) return
  const protocol: LinkProtocol = value === 'udp' ? 'udp' : 'tcp'
  void save({ protocol })
}

async function reset() {
  if (!loaded.value || link.busy.resetting) return
  const modal = useModal()
  if (modal.isOpen.value) return
  const result = await modal.open({
    title: pclText('Setup.Left.Reset.Title'),
    content: pclText('Setup.Left.Reset.GameLink.Message'),
    buttons: [
      {
        type: 'tint',
        content: pclText('Common.Action.Confirm'),
        operation: () => modal.close(true),
      },
      {
        type: 'warn',
        content: pclText('Common.Action.Cancel'),
        operation: () => modal.close(false),
      },
    ],
  })
  if (!result.confirmed || link.busy.resetting) return
  editVersion++
  try {
    Object.assign(form, await link.resetSettings())
    loaded.value = true
    loadError.value = null
    saveError.value = null
    sideTip.show(pclText('Setup.GameLink.Initialized'), 'success')
  } catch (error) {
    sideTip.show(linkErrorMessage(error, 'Setup.GameLink.Error.InitFailed'), 'warn')
  }
}

useNavAction('initialize', reset)

onMounted(() => {
  void load()
})
onUnmounted(() => editVersion++)
</script>

<template>
  <div class="pcl-page">
    <PErrorRetry v-if="loadError" :error="loadError" @retry="load" />
    <PCard title="EasyTier">
      <div class="setup-card-body">
        <PHint severity="warning">{{ pclText('Setup.GameLink.RestartRequired') }}</PHint>
        <PHint v-if="saveError" severity="error" closable @close="saveError = null">
          {{ saveError }}
        </PHint>
        <PLoading v-if="!loaded" :card="false" />
        <template v-else>
          <div class="settings-grid">
            <SetupItem :label="pclText('Setup.GameLink.Username')">
              <template #default="{ id }">
                <PTextBox
                  :input-id="id"
                  :title="pclText('Setup.GameLink.Username.ToolTip')"
                  v-model="form.username"
                  :disabled="link.busy.resetting"
                  @change="save()"
                />
              </template>
            </SetupItem>
            <SetupItem :label="pclText('Setup.GameLink.Protocol.Preference')">
              <template #default="{ id }">
                <PComboBox
                  :input-id="id"
                  :options="protocolOptions"
                  :model-value="form.protocol"
                  :disabled="link.busy.resetting"
                  @update:model-value="setProtocol"
                />
              </template>
            </SetupItem>
          </div>
          <div class="setup-checks">
            <PCheckBox
              id="PageSetupGameLink-CheckLatencyFirstMode"
              :title="pclText('Setup.GameLink.LatencyFirstMode.ToolTip')"
              :label="pclText('Setup.GameLink.LatencyFirstMode')"
              :model-value="form.latencyFirst"
              :disabled="link.busy.resetting"
              @update:model-value="save({ latencyFirst: $event })"
            />
            <PCheckBox
              id="PageSetupGameLink-CheckTryPunchSym"
              :title="pclText('Setup.GameLink.PortGuessNat.ToolTip')"
              :label="pclText('Setup.GameLink.PortGuessNat')"
              :model-value="form.tryPunchSym"
              :disabled="link.busy.resetting"
              @update:model-value="save({ tryPunchSym: $event })"
            />
            <PCheckBox
              id="PageSetupGameLink-CheckEnableIPv6"
              :title="pclText('Setup.GameLink.EnableIPv6.ToolTip')"
              :label="pclText('Setup.GameLink.EnableIPv6')"
              :model-value="form.enableIpv6"
              :disabled="link.busy.resetting"
              @update:model-value="save({ enableIpv6: $event })"
            />
            <PCheckBox
              id="PageSetupGameLink-CheckEnableCliOutput"
              :title="pclText('Setup.GameLink.CliDebugOutput.ToolTip')"
              :label="pclText('Setup.GameLink.CliDebugOutput')"
              :model-value="form.enableCliOutput"
              :disabled="link.busy.resetting"
              @update:model-value="save({ enableCliOutput: $event })"
            />
          </div>
        </template>
      </div>
    </PCard>
  </div>
</template>

<style scoped>
.setup-card-body {
  margin: 0 25px 15px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.setup-checks {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 4px;
}
</style>
