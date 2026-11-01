<script setup lang="ts">
import PTextBox from '@/components/widget/PTextBox.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import PPath from '@/components/widget/PPath.vue'
import PCard from '@/components/widget/PCard.vue'
import PButton from '@/components/widget/PButton.vue'
import PIconTextButton from '@/components/widget/PIconTextButton.vue'
import PLoading from '@/components/widget/PLoading.vue'

import PComboBox from '@/components/widget/PComboBox.vue'
import PListItem from '@/components/widget/PListItem.vue'
import sideTip from '@/composables/sideTip'
import { useModal } from '@/composables/useModal'
import useTerracottaStore from '@/stores/terracotta'
import { isTauri } from '@tauri-apps/api/core'
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAccountInfo } from '@/stores/account'
import { openExternal } from '@/util/externalLink'
const props = defineProps<{
  connected?: boolean
  connectionType?: string
  latency?: string
  versionMismatch?: boolean
}>()
const operation = ref<'create' | 'join' | ''>('')
const router = useRouter()
const route = useRoute()
const account = useAccountInfo()
const terracotta = useTerracottaStore()
const values = reactive({ TextJoinLobbyId: '' })
const selectedPort = ref('')
const error = ref('')
const roomCode = computed(() => String(route.query.code || terracotta.room || ''))
const options = computed(() =>
  terracotta.avaliable_mc_ports.map((port) => ({ key: String(port), text: String(port) })),
)
watch(options, (list) => {
  if (!list.some((option) => option.key === selectedPort.value))
    selectedPort.value = list[0]?.key || ''
})
async function connect(code: string) {
  error.value = ''
  code = code.trim()
  if (!code) {
    sideTip.show(pclText('Proto.Lobby.EnterCode'), 'warn')
    return
  }
  if (!isTauri()) {
    sideTip.show(pclText('Proto.Desktop.Link'), 'warn')
    return
  }
  operation.value = 'join'
  try {
    if (await terracotta.setGuesting(code, account.username || 'PCL.Proto Anonymous Guest'))
      await router.push({ path: '/tools/lobby/inner', query: { code } })
    else error.value = pclText('Proto.Lobby.JoinFailed')
  } finally {
    operation.value = ''
  }
}
async function create(port: string) {
  error.value = ''
  const n = Number(port)
  if (!/^\d+$/.test(port) || !Number.isInteger(n) || n < 1 || n > 65535) {
    sideTip.show(pclText('Proto.Lobby.InvalidPort'), 'warn')
    return
  }
  if (!isTauri()) {
    sideTip.show(pclText('Proto.Desktop.Link'), 'warn')
    return
  }
  operation.value = 'create'
  try {
    const code = await terracotta.setHostStarting(n, account.username || 'PCL.Proto Anonymous Host')
    await router.push({ path: '/tools/lobby/inner', query: { code } })
  } catch (err) {
    error.value = pclText('Proto.Lobby.CreateFailed', [err])
  } finally {
    operation.value = ''
  }
}
async function copy(text: string) {
  error.value = ''
  try {
    await navigator.clipboard.writeText(text)
    sideTip.show(pclText('Proto.Clipboard.CopySuccess'), 'success')
  } catch {
    sideTip.show(pclText('Proto.Clipboard.CopyFailed'), 'warn')
  }
}
async function refreshPorts() {
  error.value = ''
  if (!(await terracotta.setHostScanning())) error.value = pclText('Proto.Lobby.ScanFailed')
}
function clearLobbyId() {
  error.value = ''
  values.TextJoinLobbyId = ''
}
async function pasteLobbyId() {
  error.value = ''
  try {
    values.TextJoinLobbyId = await navigator.clipboard.readText()
  } catch {
    sideTip.show(pclText('Proto.Clipboard.ReadLobbyFailed'), 'warn')
  }
}
async function inputPort() {
  error.value = ''
  const modal = useModal()
  const result = await modal.open({
    title: pclText('Proto.Lobby.EnterPort'),
    showInput: true,
    buttons: [
      {
        type: 'tint',
        content: pclText('Launch.Account.Offline.Create'),
        operation: () => modal.close(true),
      },
      {
        type: 'default',
        content: pclText('Common.Action.Cancel'),
        operation: () => modal.close(false),
      },
    ],
  })
  if (result.confirmed && result.input) await create(result.input)
}
async function copyIp() {
  error.value = ''
  if (terracotta.url) await copy(terracotta.url)
  else sideTip.show(pclText('Proto.Lobby.NoVirtualIP'), 'warn')
}
async function leave() {
  error.value = ''
  try {
    await terracotta.setWaiting()
    await router.push('/tools/lobby')
  } catch (err) {
    error.value = pclText('Proto.Lobby.LeaveFailed', [err])
  }
}
function openWebsite(url: string) {
  error.value = ''
  void openExternal(url)
}
function disableLobby() {
  error.value = ''
  sideTip.show(pclText('Proto.NotImplemented.Action'))
}
let disposed = false
onMounted(async () => {
  if (!isTauri()) return
  await terracotta.update()
  if (disposed) return
  if (!props.connected && ['host-ok', 'guest-ok'].includes(terracotta.state)) {
    await router.replace({ path: '/tools/lobby/inner', query: { code: terracotta.room } })
    return
  }
  terracotta.startAutoUpdate()
})
onUnmounted(() => {
  disposed = true
  terracotta.stopAutoUpdate()
})

import { pclText } from '@/api/pclLocale'
const text = pclText
</script>
<template>
  <section class="pcl-page">
    <div>
      <div style="display: grid">
        <div data-control="PanContent" style="grid-row: 1 / span 1; grid-column: 1 / span 1">
          <div style="margin: 10px; display: grid; grid-template-rows: auto minmax(0, 1fr) auto">
            <div
              style="
                min-height: 36px;
                max-height: 64px;
                margin: 10px;
                grid-row: 1 / span 1;
                grid-column: 1 / span 1;
                justify-self: stretch;
              "
            >
              <span>{{
                error || (isTauri() ? pclText('Proto.Lobby.Title') : pclText('Proto.Desktop.Lobby'))
              }}</span>
            </div>
            <div
              data-control="PanSelect"
              v-if="!connected && !operation"
              style="
                grid-row: 2 / span 1;
                grid-column: 1 / span 1;
                justify-self: stretch;
                display: flex;
                flex-direction: column;
              "
            >
              <div
                style="margin: 4px 10px; display: flex; flex-direction: row; align-items: center"
              >
                <PCard data-control="BtnNatTest" style="margin: 0px 12px" hide-title>
                  <div
                    style="display: flex; flex-direction: row; align-items: center"
                    :title="text('Tools.GameLink.Nat.Title')"
                  >
                    <PSvgIcon
                      style="width: 14px; margin: 0px 7px 0px 12px"
                      name="lucide/ethernet-port"
                    />
                    <span
                      data-control="LabNatType"
                      style="margin: 0px 12px 0px 0px; color: var(--color-text-grey)"
                      class="pcl-text"
                      >{{ text('Tools.GameLink.Nat.Test') }}</span
                    >
                  </div>
                </PCard>
                <PCard data-control="BtnNatayarkUserName" hide-title>
                  <div
                    style="display: flex; flex-direction: row; align-items: center"
                    :title="text('Tools.GameLink.Natayark.Account')"
                  >
                    <PSvgIcon style="width: 14px; margin: 0px 7px 0px 12px" name="lucide/user" />
                    <div style="margin: 0px 12px 0px 0px; color: var(--color-text-grey)">
                      <span>{{ account.username || text('Tools.GameLink.Natayark.Login') }}</span>
                    </div>
                  </div>
                </PCard>
              </div>
              <PCard
                data-control="BtnSelectJoin"
                style="margin: 10px"
                :title="text('Tools.GameLink.Join.Title')"
              >
                <div style="margin: 0px 10px 10px 10px; display: flex; flex-direction: column">
                  <span style="margin: 10px 10px -2px 10px; font-size: 14px" class="pcl-text">{{
                    text('Tools.GameLink.Join.Description')
                  }}</span>
                  <div style="height: 28px; margin: 10px; display: flex; flex-direction: column">
                    <PButton
                      data-control="BtnJoin"
                      style="width: 50px; margin: 0px 0px 0px 10px"
                      @click="connect(values.TextJoinLobbyId)"
                      type="tint"
                      >{{ text('Tools.GameLink.Join.Action') }}</PButton
                    >
                    <PButton
                      data-control="BtnPaste"
                      style="width: 50px; margin: 0px 0px 0px 10px"
                      @click="pasteLobbyId"
                      >{{ text('Tools.GameLink.Join.Paste') }}</PButton
                    >
                    <PButton
                      data-control="BtnClearLobbyId"
                      style="width: 50px; margin: 0px 0px 0px 10px"
                      @click="clearLobbyId"
                      >{{ text('Tools.GameLink.Join.Clear') }}</PButton
                    >
                    <PTextBox
                      data-control="TextJoinLobbyId"
                      :placeholder="text('Tools.GameLink.Join.IdHint')"
                      v-model="values.TextJoinLobbyId"
                    />
                  </div>
                </div>
              </PCard>
              <PCard
                data-control="BtnSelectCreate"
                style="margin: 10px"
                :title="text('Tools.GameLink.Create.Title')"
                default-fold-status="unfold"
              >
                <div style="margin: 0px 10px 10px 10px; display: flex; flex-direction: column">
                  <span style="margin: 10px 10px 0px 10px; font-size: 14px" class="pcl-text">{{
                    text('Tools.GameLink.Create.Description')
                  }}</span>
                  <div style="height: 28px; margin: 10px; display: flex; flex-direction: column">
                    <PButton
                      data-control="BtnCreate"
                      style="width: 50px; margin: 0px 0px 0px 10px"
                      @click="create(selectedPort)"
                      type="tint"
                      >{{ text('Tools.GameLink.Create.Action') }}</PButton
                    >
                    <PButton
                      data-control="BtnRefresh"
                      style="width: 50px; margin: 0px 0px 0px 10px"
                      @click="refreshPorts"
                      >{{ text('Common.Action.Refresh') }}</PButton
                    >
                    <PButton
                      data-control="BtnInputPort"
                      style="width: 80px; margin: 0px 0px 0px 10px"
                      @click="inputPort"
                      >{{ text('Tools.GameLink.Create.ManualInput') }}</PButton
                    >
                    <div><PComboBox :options="options" v-model="selectedPort" /></div>
                  </div>
                </div>
              </PCard>
            </div>
            <PCard
              data-control="PanFooter"
              style="margin: 10px; grid-row: 3 / span 1; grid-column: 1 / span 1"
              hide-title
            >
              <div style="margin: 8px; opacity: 0.75; display: flex; flex-direction: column">
                <div
                  style="margin: 6px 15px; display: flex; flex-direction: row; align-items: center"
                >
                  <PButton @click="openWebsite('https://qm.qq.com/q/zfml1KaPWS')" inline>{{
                    text('Tools.GameLink.Link.Report')
                  }}</PButton>
                  <span class="pcl-text">{{ text(' | ') }}</span>
                  <PButton @click="openWebsite('https://account.naids.com/policy')" inline>{{
                    text('Tools.GameLink.Link.Terms')
                  }}</PButton>
                  <span class="pcl-text">{{ text(' | ') }}</span>
                  <PButton
                    @click="openWebsite('https://www.pclc.cc/privacy/personal-info-brief.html')"
                    inline
                    >{{ text('Tools.GameLink.Link.Privacy') }}</PButton
                  >
                  <span class="pcl-text">{{ text(' | ') }}</span>
                  <PButton data-control="BtnEulaStop" @click="disableLobby" inline>{{
                    text('Tools.GameLink.Link.Disable')
                  }}</PButton>
                </div>
                <div
                  style="margin: 6px 15px; display: flex; flex-direction: row; align-items: center"
                >
                  <span class="pcl-text">{{ text('Tools.GameLink.Link.Friends') }}</span>
                  <PButton @click="openWebsite('https://easytier.cn/')" inline>{{
                    text('Tools.GameLink.Link.EasyTier')
                  }}</PButton>
                  <span class="pcl-text">{{ text(' | ') }}</span>
                  <PButton @click="openWebsite('https://pysio.online/')" inline>{{
                    text("Pysio's Home")
                  }}</PButton>
                </div>
              </div>
            </PCard>
            <PCard v-if="operation" hide-title style="margin: 25px"
              ><PLoading
                :card="false"
                :loading-text="
                  text(
                    operation === 'create'
                      ? 'Tools.GameLink.Loading.Creating'
                      : 'Tools.GameLink.Loading.ConnectingServer',
                  )
                "
            /></PCard>
            <div
              data-control="PanFinish"
              v-if="connected"
              style="
                grid-row: 2 / span 1;
                grid-column: 1 / span 1;
                display: grid;
                grid-template-columns: auto minmax(0, 1fr);
                grid-template-rows: auto auto auto auto auto minmax(0, 1fr);
              "
            >
              <div
                style="
                  grid-row: 1 / span 1;
                  grid-column: 1 / span 2;
                  justify-self: center;
                  align-self: center;
                  display: flex;
                  flex-direction: column;
                "
              >
                <p v-if="versionMismatch" class="pcl-text">
                  {{ text('Tools.GameLink.EasyTierVersion') }}
                </p>
              </div>
              <PCard
                data-control="PanNetInfo"
                style="margin: 10px; grid-row: 3 / span 1; grid-column: 1 / span 1"
                :title="text('Tools.GameLink.Finish.InfoTitle')"
              >
                <div style="margin: 0px 10px 10px 10px; display: flex; flex-direction: column">
                  <p v-if="latency" :title="text('Tools.GameLink.Finish.PingToHost')">
                    {{ latency }}
                  </p>
                  <p v-if="connectionType" :title="text('Tools.GameLink.Finish.ConnectionType')">
                    {{ connectionType }}
                  </p>
                  <div
                    data-control="BtnFinishQuality"
                    style="margin: 5px; display: flex; flex-direction: row; align-items: center"
                    :title="text('Tools.GameLink.Finish.ConnectionStatus')"
                  >
                    <PPath
                      style="width: 14px; margin: 0px 7px 0px 0px"
                      data="M512 128c212.064 0 384 171.936 384 384s-173.632 384-384 384C299.936 896 128 724.064 128 512S299.936 128 512 128z m-126.4 575.968c-1.728 40.8 3.52 76.8 36.32 115.2 28.576 8.32 58.816 12.832 90.08 12.832 81.984 0 156.8-30.848 213.408-81.536-77.12 7.2-161.6-5.952-289.216-61.856-11.616 7.52-18.976 12.608-50.592 15.36z m-193.44-202.368c0 123.744 53.536 214.272 135.744 272.16-6.08-25.6-8.352-48.448-4.96-91.68-21.344-17.6-34.944-39.264-34.624-82.08-50.816-42.88-82.88-75.712-96.16-98.4zM544 448c-14.944 0-29.12-3.424-41.728-9.504-10.976 12.96-21.376 26.432-31.2 40.512-4.832 6.912-15.36 23.776-31.616 50.624a95.872 95.872 0 0 1 36.384 106.432c155.52 68.16 302.56 62.048 325.952 11.904 23.36-50.112-35.968-139.776-189.504-228.48A95.68 95.68 0 0 1 544 448z m-160 128a32 32 0 1 0 0 64 32 32 0 0 0 0-64z m-161.92-199.2c-16.576 35.52 16.96 99.68 94.08 163.2a95.936 95.936 0 0 1 59.616-27.648c19.936-35.008 30.72-52.736 42.88-70.08 10.944-15.648 22.528-30.656 38.432-49.44-5.696-12.128-8.96-25.6-9.088-39.872-118.176-30.4-206.88-16.96-225.92 23.84z m415.136-47.84c1.824 7.36 3.296 19.104 2.336 32.416 88.96 48.704 153.76 105.056 192.32 160.704 0-118.656-34.208-176.064-75.52-210.976-49.792-5.92-69.952-4.064-119.136 17.856zM544 320a32 32 0 1 0 0 64 32 32 0 0 0 0-64z m-32-128c-82.016 0-156.8 30.848-213.44 81.6 47.264-4.384 96-0.544 170.304 18.624A95.84 95.84 0 0 1 544 256c21.568 0 41.472 7.104 57.504 19.104 35.968-17.28 62.016-25.824 87.136-29.984A318.528 318.528 0 0 0 512 192z"
                    />
                    <div style="margin: 0px 12px 0px 0px; color: var(--color-text-grey)">
                      <span>{{
                        text(
                          ['host-ok', 'guest-ok'].includes(terracotta.state)
                            ? 'Tools.GameLink.Finish.Connected'
                            : 'Tools.GameLink.Finish.Connecting',
                        )
                      }}</span>
                    </div>
                  </div>
                  <div
                    data-control="BtnFinishId"
                    style="margin: 5px; display: flex; flex-direction: row; align-items: center"
                    :title="text('Tools.GameLink.Finish.LobbyId')"
                  >
                    <PPath
                      style="width: 14px; margin: 0px 7px 0px 0px"
                      data="M21.15,2.86a2.89,2.89,0,0,0-3-.71L4,6.88a2.9,2.9,0,0,0-.12,5.47l5.24,2h0a.93.93,0,0,1,.53.52l2,5.25A2.87,2.87,0,0,0,14.36,22h.07a2.88,2.88,0,0,0,2.69-2L21.85,5.83A2.89,2.89,0,0,0,21.15,2.86ZM20,5.2,15.22,19.38a.88.88,0,0,1-.84.62.92.92,0,0,1-.87-.58l-2-5.25a2.91,2.91,0,0,0-1.67-1.68l-5.25-2A.9.9,0,0,1,4,9.62a.88.88,0,0,1,.62-.84L18.8,4.05A.91.91,0,0,1,20,5.2Z"
                    />
                    <div style="margin: 0px 12px 0px 0px; color: var(--color-text-grey)">
                      <span>{{ roomCode }}</span>
                    </div>
                  </div>
                </div>
              </PCard>
              <PCard style="margin: 10px; grid-row: 2 / span 1; grid-column: 1 / span 1" hide-title>
                <div style="display: flex; flex-direction: row; align-items: center">
                  <div
                    data-control="BtnConnectUserName"
                    style="display: flex; flex-direction: row; align-items: center"
                    :title="text('Tools.GameLink.Finish.UserName')"
                  >
                    <PPath
                      style="width: 14px; margin: 0px 7px 0px 12px"
                      data="M512 447.223c-88.224 0-160-71.776-160-160s71.776-160 160-160c88.225 0 160 71.776 160 160s-71.775 160-160 160z m0-256c-52.935 0-96 43.065-96 96s43.065 96 96 96 96-43.065 96-96-43.065-96-96-96zM454.901 870.594c-96.594 0-184.933-3.802-231.263-49.955C203.308 800.386 193 774.164 193 742.701c0-31.629 10.247-62.812 30.457-92.686 17.978-26.573 42.908-50.741 74.098-71.833C359.256 536.46 437.418 512.53 512 512.53c74.55 0 152.55 23.943 214.002 65.691 31.05 21.094 55.861 45.273 73.746 71.867C819.822 679.937 830 711.096 830 742.701c0 31.552-10.317 57.827-30.664 78.097-50.714 50.521-151.822 50.128-258.88 49.723a7395.45 7395.45 0 0 0-56.914-0.001c-9.605 0.037-19.163 0.074-28.641 0.074zM512 806.447c9.567 0 19.149 0.037 28.701 0.073 49.52 0.191 96.284 0.37 135.808-4.396 38.418-4.633 64.546-13.604 77.659-26.668 5.079-5.06 11.832-13.96 11.832-32.755 0-38.089-27.688-78.744-75.963-111.54C638.933 596.442 574.04 576.53 512 576.53c-126.309 0-255 83.862-255 166.171 0 18.675 6.738 27.547 11.807 32.596 32.045 31.922 128.975 31.55 214.491 31.224 9.556-0.037 19.139-0.074 28.702-0.074z"
                    />
                    <div style="margin: 0px 12px 0px 0px; color: var(--color-text-grey)">
                      <span>{{ terracotta.username }}</span>
                    </div>
                  </div>
                  <span
                    data-control="SplitLineBeforeUserType"
                    style="margin: 8.8px 0px 8px 0px; opacity: 0.3"
                    class="pcl-text"
                    >{{ text('|') }}</span
                  >
                  <div
                    data-control="BtnConnectUserType"
                    style="display: flex; flex-direction: row; align-items: center"
                    :title="text('Tools.GameLink.Finish.UserType')"
                  >
                    <PPath
                      style="width: 13px; height: 13px; margin: 0px 7px 0px 12px"
                      data="M274.773333 419.498667h154.624v76.8H274.773333v-76.8z m0-154.965334h399.701334V341.333333H274.773333V264.533333z M92.16 105.472v832.170667h179.541333v-76.8H168.96V182.272h678.229333v227.669333h76.8V105.472H92.16z M645.12 720.213333c91.818667-1.024 165.888-75.776 165.888-167.936 0-92.842667-75.093333-167.936-167.936-167.936s-167.936 75.093333-167.936 167.936c0 92.16 74.069333 166.912 165.888 167.936-129.365333 1.024-238.933333 90.794667-279.210667 215.722667h81.92c35.84-81.92 111.616-138.922667 198.997334-138.922667s163.498667 56.661333 198.997333 138.922667h82.261333c-40.277333-124.586667-149.845333-214.698667-278.869333-215.722667z m-2.389333-76.8c-50.176 0-91.477333-40.96-91.477334-91.477333 0-50.176 40.96-91.477333 91.477334-91.477333 50.176 0 91.477333 40.96 91.477333 91.477333s-40.96 91.477333-91.477333 91.477333z"
                    />
                    <div style="margin: 0px 12px 0px 0px; color: var(--color-text-grey)">
                      <span>{{
                        text(
                          terracotta.state.startsWith('host')
                            ? 'Tools.GameLink.Finish.Host'
                            : 'Tools.GameLink.Finish.Guest',
                        )
                      }}</span>
                    </div>
                  </div>
                </div>
              </PCard>
              <PCard
                data-control="PanOperation"
                style="margin: 10px; grid-row: 4 / span 1; grid-column: 1 / span 1"
                :title="text('Tools.GameLink.Finish.Actions')"
              >
                <div style="margin: 0px 10px 10px 10px; display: flex; flex-direction: column">
                  <PIconTextButton
                    data-control="BtnFinishCopyIp"
                    icon="lucide/copy"
                    style="margin: 1px -2px"
                    @click="copyIp"
                    >{{ text('Tools.GameLink.Finish.CopyVirtualIp') }}</PIconTextButton
                  >
                  <PIconTextButton
                    data-control="BtnFinishCopy"
                    icon="lucide/copy"
                    style="margin: 1px -2px"
                    @click="copy(roomCode)"
                    >{{ text('Tools.GameLink.Finish.CopyLobbyId') }}</PIconTextButton
                  >
                  <PIconTextButton
                    data-control="BtnFinishExit"
                    icon="lucide/power"
                    style="margin: 1px -2px"
                    @click="leave"
                    >{{ text('Tools.GameLink.Finish.Exit') }}</PIconTextButton
                  >
                </div>
              </PCard>
              <PCard
                data-control="CardPlayerList"
                style="
                  min-width: 300px;
                  margin: 10px;
                  grid-row: 2 / span 4;
                  grid-column: 2 / span 1;
                "
                :title="
                  terracotta.profiles
                    ? text('Tools.GameLink.Member.ListCount', [terracotta.profiles.length])
                    : text('Tools.GameLink.Member.ListLoading')
                "
              >
                <div style="margin: 0px 6px 8px 5px; display: flex; flex-direction: column">
                  <PListItem
                    v-for="profile in terracotta.profiles"
                    :key="profile.machine_id"
                    :title="profile.name"
                    :subtitle="`${profile.kind} · ${profile.vendor}`"
                  />
                </div>
              </PCard>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
