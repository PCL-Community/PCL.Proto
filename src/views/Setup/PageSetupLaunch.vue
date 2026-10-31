<script setup lang="ts">
import SetupItem from '@/components/widget/SetupItem.vue'
import { useSetup } from '@/stores/setup'
import { watch, computed, onMounted, ref } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { useJavaStore } from '@/stores/java'
import { useRouter } from 'vue-router'
import { pclText } from '@/api/pclLocale'
import { usePageSettings } from '@/composables/usePageSettings'
import PButton from '@/components/widget/PButton.vue'
import PPrimaryAction from '@/components/widget/PPrimaryAction.vue'
import PCard from '@/components/widget/PCard.vue'
import PCheckBox from '@/components/widget/PCheckBox.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PHint from '@/components/widget/PHint.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import PTextBox from '@/components/widget/PTextBox.vue'
import {
  windowSettingKeys,
  windowSaveError,
  readWindowSettings,
  saveWindowSettings,
  type WindowSettings,
} from '@/api/windowSettings'
import { memorySettingKeys, memoryTickToMib } from '@/api/memorySettings'
import { memoryGiB, useMemorySettings } from '@/composables/useMemorySettings'
import sideTip from '@/composables/sideTip'
import { getIsolationDefault, setIsolationDefault, isolationChoices } from '@/api/isolationSettings'
import { launchArgumentKeys } from '@/api/launchArguments'
import { useLaunchArguments } from '@/composables/useLaunchArguments'
import { invoke } from '@tauri-apps/api/core'
import sourceDefaults from '@/assets/setting-defaults.json'
const setup = useSetup()
const props = defineProps<{ java32Bit?: boolean; memoryOverLimit?: boolean }>()
const java = useJavaStore()
const java32Bit = computed(
  () => props.java32Bit ?? (java.runtimes.length > 0 && java.globalIs64Bit === false),
)
onMounted(() => {
  if (isTauri())
    void java.load().catch((error) => console.warn('Unable to read Java memory limits', error))
})
const desktop = isTauri()
const { values } = usePageSettings(
  'PageSetupLaunch',
  {
    TextArgumentTitle: setup.launch.gameWindowTitle,
    TextArgumentInfo: setup.launch.customInfo,
    ComboArgumentVisibie: '4',
    ComboArgumentPriority: '3',
    ComboArgumentWindowType: '1',
    ComboPreferredIpStack: '0',
  },
  desktop
    ? [
        ...windowSettingKeys,
        ...memorySettingKeys,
        ...launchArgumentKeys,
        'ComboArgumentIndieV2',
        'ComboArgumentVisibie',
      ]
    : [],
)
type Visibility = 'hide_and_exit' | 'hide_and_reopen' | 'minimize' | 'keep'
const visibilityChoices: Record<string, Visibility> = {
  '1': 'hide_and_exit',
  '2': 'hide_and_reopen',
  '3': 'minimize',
  '4': 'keep',
}
const visibilityReady = ref(false)
const visibilityBusy = ref(false)
const visibilityError = ref('')
const visibilityValue = ref('4')
const visibilityRevision = ref(0)
let visibilityDraft: string | undefined
async function loadVisibility() {
  try {
    const mode = await invoke<Visibility>('get_launcher_visibility')
    const key = Object.keys(visibilityChoices).find((key) => visibilityChoices[key] === mode)
    if (!key) throw new Error('Unknown launcher visibility')
    visibilityValue.value = key
    visibilityReady.value = true
    visibilityError.value = ''
  } catch (error) {
    visibilityError.value = String(error)
  }
}
async function changeVisibility(key: string | undefined) {
  const mode = key && visibilityChoices[key]
  if (!mode || visibilityBusy.value || !visibilityReady.value) return
  visibilityDraft = key
  visibilityBusy.value = true
  try {
    await invoke('set_launcher_visibility', { mode })
    visibilityValue.value = key
    visibilityDraft = undefined
    visibilityError.value = ''
  } catch (error) {
    visibilityError.value = String(error)
  } finally {
    visibilityBusy.value = false
    visibilityRevision.value++
  }
}
if (desktop) void loadVisibility()
const {
  jvm: argumentJvm,
  game: argumentGame,
  ready: argumentsReady,
  error: argumentsError,
  load: loadArguments,
  save: saveArguments,
} = useLaunchArguments()
const isolationValue = ref('4')
const isolationReady = ref(false)
const isolationBusy = ref(false)
const isolationError = ref('')
const isolationControlRevision = ref(0)
async function loadIsolation() {
  try {
    const value = await getIsolationDefault()
    const index = isolationChoices.indexOf(value)
    if (index < 0) throw new Error('Unknown isolation setting')
    isolationValue.value = String(index)
    isolationReady.value = true
    isolationError.value = ''
  } catch (error) {
    isolationError.value = String(error)
  }
}
async function changeIsolation(value: string | undefined) {
  const setting = isolationChoices[Number(value)]
  if (!setting || isolationBusy.value || !isolationReady.value) return
  isolationBusy.value = true
  try {
    await setIsolationDefault(setting)
    isolationValue.value = String(isolationChoices.indexOf(setting))
    isolationError.value = ''
  } catch (error) {
    isolationError.value = String(error)
  } finally {
    isolationBusy.value = false
    isolationControlRevision.value++
  }
}
if (desktop) void loadIsolation()
const memoryJava64 = computed(() => (java.runtimes.length ? java.globalIs64Bit : null))
const {
  mode: memoryMode,
  maximum: memoryMaximum,
  initial: memoryInitial,
  ready: memoryReady,
  error: memoryError,
  preview: memoryPreview,
  segments: memorySegments,
  load: loadMemory,
  save: saveMemory,
} = useMemorySettings(undefined, memoryJava64)

const windowReady = ref(false)
const windowError = ref('')
let hydratingWindow = false
function loadWindowSettings() {
  void readWindowSettings()
    .then((settings) => {
      hydratingWindow = true
      values.ComboArgumentWindowType = { default: '1', launcher: '2', custom: '3' }[settings.mode]
      values.TextArgumentWindowWidth = String(settings.width)
      values.TextArgumentWindowHeight = String(settings.height)
      hydratingWindow = false
      windowReady.value = true
      windowError.value = ''
    })
    .catch((error) => {
      windowError.value = String(error)
    })
}
function applyWindowSettings() {
  if (!windowReady.value) return
  const width = Number(values.TextArgumentWindowWidth)
  const height = Number(values.TextArgumentWindowHeight)
  if (
    !/^\d+$/.test(values.TextArgumentWindowWidth) ||
    !/^\d+$/.test(values.TextArgumentWindowHeight) ||
    !Number.isInteger(width) ||
    !Number.isInteger(height) ||
    width < 100 ||
    height < 100 ||
    width > 16384 ||
    height > 16384
  ) {
    windowError.value = pclText('Proto.Settings.InvalidWindowSize')
    return
  }
  const mode = (
    { '1': 'default', '2': 'launcher', '3': 'custom' } as Record<string, WindowSettings['mode']>
  )[values.ComboArgumentWindowType]
  if (!mode) return
  windowError.value = ''
  void saveWindowSettings({ mode, width, height }).catch((error) =>
    sideTip.show(String(error), 'warn'),
  )
}
if (desktop) loadWindowSettings()
watch(
  () => values.ComboArgumentWindowType,
  () => {
    if (!hydratingWindow) applyWindowSettings()
  },
  { flush: 'sync' },
)

watch(
  () => [values.TextArgumentTitle, values.TextArgumentInfo],
  ([title, info]) => {
    setup.launch.gameWindowTitle = title || ''
    setup.launch.customInfo = info || ''
  },
  { immediate: true },
)
const router = useRouter()
const text = pclText
</script>
<template>
  <div class="pcl-page">
    <p class="settings-unavailable">{{ text('Proto.Settings.Unavailable') }}</p>
    <p v-if="visibilityError" role="alert">
      {{ visibilityError }}
      <button @click="visibilityReady ? changeVisibility(visibilityDraft) : loadVisibility()">
        {{ text('Launch.Homepage.LoadFailed.Retry') }}
      </button>
    </p>
    <p v-if="isolationError" role="alert">
      {{ isolationError }}
      <button @click="loadIsolation">{{ text('Launch.Homepage.LoadFailed.Retry') }}</button>
    </p>
    <p v-if="argumentsError" role="alert">
      {{ argumentsError }}
      <button @click="argumentsReady ? saveArguments() : loadArguments()">
        {{ text('Launch.Homepage.LoadFailed.Retry') }}
      </button>
    </p>
    <p v-if="windowError || windowSaveError" role="alert">
      {{ windowError || windowSaveError }}
      <button @click="windowReady ? applyWindowSettings() : loadWindowSettings()">
        {{ text('Launch.Homepage.LoadFailed.Retry') }}
      </button>
    </p>
    <div>
      <div data-control="PanBack">
        <div
          data-control="PanMain"
          style="margin: 25px 25px 10px 25px; display: flex; flex-direction: column"
        >
          <PCard
            data-control="CardArgument"
            style="margin: 0px 0px 15px 0px"
            :title="text('Setup.Launch.Options.Title')"
          >
            <div style="margin: 0px 25px 15px 25px; display: flex; flex-direction: column">
              <div class="settings-grid">
                <SetupItem
                  :label="text('Setup.Launch.Options.InstanceIsolation.Label')"
                  v-slot="{ id }"
                  ><PComboBox
                    :disabled="!desktop || !isolationReady || isolationBusy"
                    :key="isolationControlRevision"
                    :input-id="id"
                    data-control="ComboArgumentIndieV2"
                    :title="text('Setup.Launch.Options.InstanceIsolation.ToolTip')"
                    :options="[
                      {
                        key: '0',
                        text: text('Common.Action.Close'),
                        title: text('Setup.Launch.Options.InstanceIsolation.None.ToolTip'),
                      },
                      {
                        key: '1',
                        text: text('Setup.Launch.Options.InstanceIsolation.ModdedOnly'),
                        title: text('Setup.Launch.Options.InstanceIsolation.ModdedOnly.ToolTip'),
                      },
                      {
                        key: '2',
                        text: text('Setup.Launch.Options.InstanceIsolation.NonRelease'),
                        title: text('Setup.Launch.Options.InstanceIsolation.NonRelease.ToolTip'),
                      },
                      {
                        key: '3',
                        text: text('Setup.Launch.Options.InstanceIsolation.ModdedAndNonRelease'),
                      },
                      {
                        key: '4',
                        text: text('Setup.Launch.Options.InstanceIsolation.All'),
                        title: text('Setup.Launch.Options.InstanceIsolation.All.ToolTip'),
                      },
                    ]"
                    :model-value="isolationValue"
                    @update:model-value="changeIsolation"
                /></SetupItem>
                <SetupItem :label="text('Setup.Launch.Options.WindowTitle.Label')" v-slot="{ id }"
                  ><PTextBox
                    disabled
                    :input-id="id"
                    data-control="TextArgumentTitle"
                    :suggestions="[text('Setup.Launch.Options.WindowTitle.SampleFormat')]"
                    :title="text('Setup.Launch.Options.WindowTitle.ToolTip')"
                    :placeholder="text('Common.Option.Default')"
                    v-model="values.TextArgumentTitle"
                /></SetupItem>
                <SetupItem :label="text('Setup.Launch.Options.CustomInfo.Label')" v-slot="{ id }"
                  ><PTextBox
                    disabled
                    :input-id="id"
                    data-control="TextArgumentInfo"
                    :title="text('Setup.Launch.Options.CustomInfo.ToolTip')"
                    :placeholder="text('')"
                    v-model="values.TextArgumentInfo"
                /></SetupItem>
                <SetupItem :label="text('Setup.Launch.Options.Visibility.Label')" v-slot="{ id }"
                  ><PComboBox
                    :disabled="!desktop || !visibilityReady || visibilityBusy"
                    :key="visibilityRevision"
                    :input-id="id"
                    data-control="ComboArgumentVisibie"
                    :options="[
                      {
                        key: '0',
                        text: text('Setup.Launch.Options.Visibility.CloseImmediately'),
                        disabled: true,
                      },
                      {
                        key: '1',
                        text: text('Setup.Launch.Options.Visibility.HideAndCloseOnExit'),
                      },
                      {
                        key: '2',
                        text: text('Setup.Launch.Options.Visibility.HideAndReopenOnExit'),
                      },
                      { key: '3', text: text('Setup.Launch.Options.Visibility.Minimize') },
                      { key: '4', text: text('Setup.Launch.Options.Visibility.KeepUnchanged') },
                    ]"
                    :model-value="visibilityValue"
                    @update:model-value="changeVisibility"
                /></SetupItem>
                <SetupItem :label="text('Setup.Launch.Options.Priority.Label')" v-slot="{ id }"
                  ><PComboBox
                    disabled
                    :input-id="id"
                    data-control="ComboArgumentPriority"
                    :options="[
                      { key: '0', text: text('Setup.Launch.Options.Priority.RealTime') },
                      { key: '1', text: text('Setup.Launch.Options.Priority.High') },
                      { key: '2', text: text('Setup.Launch.Options.Priority.AboveNormal') },
                      { key: '3', text: text('Setup.Launch.Options.Priority.Normal') },
                      { key: '4', text: text('Setup.Launch.Options.Priority.BelowNormal') },
                    ]"
                    v-model="values.ComboArgumentPriority"
                /></SetupItem>
                <SetupItem :label="text('Setup.Launch.Options.WindowSize.Label')" v-slot="{ id }"
                  ><div
                    data-control="PanArgumentWindow"
                    style="display: grid; grid-template-columns: minmax(0, 1fr) auto auto auto"
                  >
                    <PComboBox
                      :disabled="!windowReady"
                      :input-id="id"
                      data-control="ComboArgumentWindowType"
                      style="grid-row: 1 / span 1; grid-column: 1 / span 1"
                      :title="text('Setup.Launch.Options.WindowSize.ModernUiWarning')"
                      :options="[
                        {
                          key: '0',
                          text: text('Setup.Launch.Options.WindowSize.Fullscreen'),
                          disabled: true,
                        },
                        { key: '1', text: text('Common.Option.Default') },
                        { key: '2', text: text('Setup.Launch.Options.WindowSize.SameAsLauncher') },
                        {
                          key: '3',
                          text: text('Setup.Launch.Options.WindowSize.Custom'),
                          title: text('Setup.Launch.Options.WindowSize.CustomHeightBug.ToolTip'),
                        },
                        {
                          key: '4',
                          text: text('Setup.Launch.Options.WindowSize.Maximized'),
                          disabled: true,
                        },
                      ]"
                      v-model="values.ComboArgumentWindowType"
                    />
                    <PTextBox
                      :disabled="!windowReady"
                      @blur="applyWindowSettings"
                      data-control="TextArgumentWindowWidth"
                      style="
                        width: 60px;
                        margin: 0px 0px 0px 50px;
                        padding: 0px 5px 0px 0px;
                        grid-row: 1 / span 1;
                        grid-column: 2 / span 1;
                      "
                      v-if="values.ComboArgumentWindowType === '3'"
                      :placeholder="text('')"
                      v-model="values.TextArgumentWindowWidth"
                    />
                    <span
                      data-control="LabArgumentWindowMiddle"
                      style="
                        font-size: 18px;
                        grid-row: 1 / span 1;
                        grid-column: 3 / span 1;
                        align-self: center;
                      "
                      v-if="values.ComboArgumentWindowType === '3'"
                      class="pcl-text"
                      >{{ text(' × ') }}</span
                    >
                    <PTextBox
                      :disabled="!windowReady"
                      @blur="applyWindowSettings"
                      data-control="TextArgumentWindowHeight"
                      style="
                        width: 60px;
                        padding: 0px 5px 0px 0px;
                        grid-row: 1 / span 1;
                        grid-column: 4 / span 1;
                      "
                      v-if="values.ComboArgumentWindowType === '3'"
                      :title="text('Setup.Launch.Options.WindowSize.CustomHeightBug.ToolTip')"
                      :placeholder="text('')"
                      v-model="values.TextArgumentWindowHeight"
                    /></div
                ></SetupItem>
                <SetupItem :label="text('Setup.Launch.Options.MsAuth.Label')" v-slot="{ id }"
                  ><PComboBox
                    disabled
                    :input-id="id"
                    data-control="ComboMsAuthType"
                    :title="text('Setup.Launch.Options.MsAuth.ToolTip')"
                    :options="[
                      { key: '0', text: text('Setup.Launch.Options.MsAuth.WebManager') },
                      { key: '1', text: text('Setup.Launch.Options.MsAuth.DeviceCode') },
                    ]"
                    v-model="values.ComboMsAuthType"
                /></SetupItem>
                <SetupItem :label="text('Setup.Launch.Options.IpStack.Label')" v-slot="{ id }"
                  ><PComboBox
                    disabled
                    :input-id="id"
                    data-control="ComboPreferredIpStack"
                    :title="text('Setup.Launch.Options.IpStack.ToolTip')"
                    :options="[
                      { key: '0', text: text('Setup.Launch.Options.IpStack.Ipv4') },
                      { key: '1', text: text('Setup.Launch.Options.IpStack.JavaDefault') },
                      { key: '2', text: text('Setup.Launch.Options.IpStack.Ipv6') },
                    ]"
                    v-model="values.ComboPreferredIpStack"
                /></SetupItem>
              </div>
            </div>
          </PCard>
          <PCard style="margin: 0px 0px 15px 0px" :title="text('Setup.Launch.Memory.Title')">
            <PHint v-if="java32Bit" severity="warning">{{
              text('Setup.Launch.Memory.Warning.Java32Bit')
            }}</PHint>
            <PHint v-if="memoryOverLimit" severity="warning">{{
              text('Setup.Launch.Memory.Warning.TooMuch')
            }}</PHint>
            <div style="margin: 0px 25px 15px 25px; display: flex; flex-direction: column">
              <div
                style="
                  margin: 0px 0px 3px 0px;
                  display: grid;
                  grid-template-columns: auto minmax(0, 1fr);
                  grid-template-rows: auto auto auto auto;
                "
              >
                <label
                  data-control="RadioRamType0"
                  style="
                    width: 110px;
                    margin: 0px 20px 9px 0px;
                    grid-row: 1 / span 1;
                    grid-column: 1 / span 1;
                    justify-self: start;
                  "
                  :title="text('Setup.Launch.Memory.Auto.ToolTip')"
                  class="pcl-radio"
                  ><input
                    :disabled="!memoryReady"
                    type="radio"
                    v-model="memoryMode"
                    value="auto"
                    @change="saveMemory"
                  />{{ text('Setup.Launch.Memory.Auto') }}</label
                >
                <label
                  data-control="RadioRamType1"
                  style="margin: 0px 20px 9px 0px; grid-row: 2 / span 1; grid-column: 1 / span 1"
                  class="pcl-radio"
                  ><input
                    :disabled="!memoryReady"
                    type="radio"
                    v-model="memoryMode"
                    value="custom"
                    @change="saveMemory"
                  />{{ text('Common.Option.Customize') }}</label
                >
                <span
                  style="
                    margin: 0px 20px 9px 0px;
                    grid-row: 3 / span 1;
                    grid-column: 1 / span 1;
                    align-self: center;
                  "
                  :title="text('Setup.Launch.Memory.Max.ToolTip')"
                  class="pcl-text"
                  >{{ text('Setup.Launch.Memory.Max')
                  }}<template v-if="memoryMode === 'custom'">
                    ({{ memoryTickToMib(memoryMaximum) }} MiB)</template
                  ></span
                >
                <input
                  data-control="SliderRamCustom"
                  :disabled="!memoryReady || memoryMode !== 'custom'"
                  style="margin: 0px 12px 9px 0px; grid-row: 3 / span 1; grid-column: 2 / span 1"
                  type="range"
                  min="0"
                  max="49"
                  v-model.number="memoryMaximum"
                  @change="saveMemory"
                />
                <span
                  style="
                    margin: 0px 20px 0px 0px;
                    grid-row: 4 / span 1;
                    grid-column: 1 / span 1;
                    align-self: center;
                  "
                  :title="text('Setup.Launch.Memory.Initial.ToolTip')"
                  class="pcl-text"
                  >{{ text('Setup.Launch.Memory.Initial')
                  }}<template v-if="memoryMode === 'custom'">
                    ({{
                      memoryInitial
                        ? `${Math.min(memoryTickToMib(memoryInitial), memoryTickToMib(memoryMaximum))} MiB`
                        : text('Setup.Launch.Memory.Auto')
                    }})</template
                  ></span
                >
                <input
                  data-control="SliderRamInitialCustom"
                  :disabled="!memoryReady || memoryMode !== 'custom'"
                  style="grid-row: 4 / span 1; grid-column: 2 / span 1"
                  :title="text('Setup.Launch.Memory.Initial.ToolTip')"
                  type="range"
                  min="0"
                  max="49"
                  v-model.number="memoryInitial"
                  @change="saveMemory"
                />
              </div>
              <p v-if="memoryError" role="alert">
                {{ memoryError }}
                <button @click="memoryReady ? saveMemory() : loadMemory()">
                  {{ text('Launch.Homepage.LoadFailed.Retry') }}
                </button>
              </p>
              <div
                data-control="PanRamDisplay"
                style="
                  margin: 11px 2px 0px 0px;
                  display: grid;
                  grid-template-columns: minmax(0, 1fr) auto;
                  grid-template-rows: minmax(0, 1fr) 4px minmax(0, 1fr);
                "
              >
                <div
                  data-control="RectRamUsed"
                  style="grid-row: 2; grid-column: 1 / span 2; justify-self: start"
                  :style="{ width: `${memorySegments.used}%` }"
                  class="pcl-divider"
                />
                <div
                  data-control="RectRamGame"
                  style="opacity: 0.5; grid-row: 2; grid-column: 1 / span 2; justify-self: start"
                  :style="{
                    width: `${memorySegments.game}%`,
                    marginLeft: `${memorySegments.used}%`,
                  }"
                  class="pcl-divider"
                />
                <div
                  data-control="RectRamEmpty"
                  style="opacity: 0.7; grid-row: 2; grid-column: 1 / span 2; justify-self: start"
                  :style="{
                    width: `${memorySegments.empty}%`,
                    marginLeft: `${memorySegments.used + memorySegments.game}%`,
                  }"
                  class="pcl-divider"
                />
                <span
                  data-control="LabRamUsedTitle"
                  style="
                    margin: 0px 0px 5px 2px;
                    font-size: 11px;
                    opacity: 0.7;
                    grid-row: 1 / span 1;
                    grid-column: 1 / span 1;
                    justify-self: start;
                  "
                  class="pcl-text"
                  >{{ text('Setup.Launch.Memory.UsedTotal') }}</span
                >
                <span
                  data-control="LabRamGameTitle"
                  style="
                    margin: 0px 0px 5px 2px;
                    font-size: 11px;
                    opacity: 0.7;
                    grid-row: 1 / span 1;
                    grid-column: 2 / span 1;
                    justify-self: end;
                  "
                  class="pcl-text"
                  >{{ text('Proto.Settings.MemoryEstimate') }}</span
                >
                <div
                  style="
                    margin: 3px 0px 0px 2px;
                    grid-row: 3 / span 1;
                    grid-column: 1 / span 1;
                    justify-self: start;
                    display: flex;
                    flex-direction: row;
                    flex-wrap: wrap;
                    min-width: 0;
                    align-items: center;
                  "
                >
                  <span data-control="LabRamUsed" style="font-size: 16px" class="pcl-text">{{
                    memoryPreview
                      ? memoryGiB(memoryPreview.total_bytes - memoryPreview.available_bytes)
                      : '—'
                  }}</span>
                  <span data-control="LabRamTotal" style="font-size: 16px" class="pcl-text">{{
                    memoryPreview ? ` / ${memoryGiB(memoryPreview.total_bytes)}` : ' / —'
                  }}</span>
                </div>
                <span
                  data-control="LabRamGame"
                  style="
                    margin: 3px 0px 0px 2px;
                    font-size: 16px;
                    grid-row: 3 / span 1;
                    grid-column: 2 / span 1;
                    justify-self: end;
                  "
                  class="pcl-text"
                  >{{
                    memoryPreview?.allocation_mib != null
                      ? memoryGiB(memoryPreview.allocation_mib * 1048576)
                      : '—'
                  }}</span
                >
              </div>
            </div>
          </PCard>
          <PCard
            style="margin: 0px 0px 15px 0px"
            :title="text('Setup.Launch.Advanced.Title')"
            default-fold-status="fold"
          >
            <div class="settings-grid" style="margin: 0px 25px 15px 25px">
              <SetupItem :label="text('Setup.Launch.Advanced.Renderer.Label')" v-slot="{ id }"
                ><PComboBox
                  disabled
                  :input-id="id"
                  data-control="ComboAdvanceRenderer"
                  :options="[
                    { key: '0', text: text('Setup.Launch.Advanced.Renderer.GameDefault') },
                    { key: '1', text: text('Setup.Launch.Advanced.Renderer.Software') },
                    { key: '2', text: text('Setup.Launch.Advanced.Renderer.DirectX12') },
                    { key: '3', text: text('Setup.Launch.Advanced.Renderer.Vulkan') },
                  ]"
                  v-model="values.ComboAdvanceRenderer"
              /></SetupItem>
              <SetupItem
                ><template #label="{ id }"
                  ><div style="display: flex; flex-direction: row; align-items: center">
                    <label :for="id" class="pcl-text">{{
                      text('Setup.Launch.Advanced.JvmHead')
                    }}</label>
                    <PButton
                      data-control="BtnAdvanceJvmReset"
                      :disabled="!desktop || !argumentsReady"
                      style="height: 23px; margin: 1px 0px 0px 0px"
                      :title="text('Setup.Launch.Advanced.JvmHead.Restore')"
                      @click="
                        argumentJvm = sourceDefaults.PageSetupLaunch.TextAdvanceJvm;
                        saveArguments()
                      "
                      inline
                      ><PSvgIcon name="lucide/rotate-ccw" :size="18"
                    /></PButton></div></template
                ><template #default="{ id }"
                  ><PTextBox
                    :disabled="!desktop || !argumentsReady"
                    :input-id="id"
                    multiline
                    data-control="TextAdvanceJvm"
                    style="height: 100px; padding: 5px 0px"
                    :title="
                      text('Setup.Launch.Advanced.JvmHead.ToolTip') +
                      '\n\n' +
                      text('Proto.LaunchArguments.Parsing') +
                      '\n' +
                      text('Proto.LaunchArguments.Hint')
                    "
                    :placeholder="text('')"
                    v-model="argumentJvm"
                    @blur="saveArguments()" /></template
              ></SetupItem>
              <SetupItem :label="text('Setup.Launch.Advanced.GameTail')" v-slot="{ id }"
                ><PTextBox
                  :disabled="!desktop || !argumentsReady"
                  :input-id="id"
                  data-control="TextAdvanceGame"
                  :title="
                    text('Setup.Launch.Advanced.GameTail.ToolTip') +
                    '\n\n' +
                    text('Proto.LaunchArguments.Parsing')
                  "
                  :placeholder="text('')"
                  v-model="argumentGame"
                  @blur="saveArguments()"
              /></SetupItem>
              <SetupItem :label="text('Setup.Launch.Advanced.PreLaunchCommand')" v-slot="{ id }"
                ><PTextBox
                  disabled
                  :input-id="id"
                  data-control="TextAdvanceRun"
                  :title="text('Setup.Launch.Advanced.PreLaunchCommand.ToolTip')"
                  :placeholder="text('')"
                  v-model="values.TextAdvanceRun"
              /></SetupItem>
              <PCheckBox
                disabled
                v-if="String(values.TextAdvanceRun || '').trim()"
                data-control="CheckAdvanceRunWait"
                style="margin: 8px 0 7px; grid-column: 2"
                id="PageSetupLaunch-CheckAdvanceRunWait"
                :label="text('Setup.Launch.Advanced.PreLaunchCommand.Wait')"
                v-model="values.CheckAdvanceRunWait"
              />
              <div
                style="
                  margin: 12px 0px 4px 0px;
                  display: flex;
                  flex-direction: column;
                  grid-column: 1 / -1;
                "
              >
                <PCheckBox
                  disabled
                  data-control="CheckAdvanceDisableJLW"
                  style="height: 28px"
                  :title="text('Setup.Launch.Advanced.DisableJlw.ToolTip')"
                  id="PageSetupLaunch-CheckAdvanceDisableJLW"
                  :label="text('Setup.Launch.Advanced.DisableJlw')"
                  v-model="values.CheckAdvanceDisableJLW"
                />
                <PCheckBox
                  disabled
                  data-control="CheckAdvanceDisableLF"
                  style="height: 28px"
                  :title="text('Setup.Launch.Advanced.DisableLF.ToolTip')"
                  id="PageSetupLaunch-CheckAdvanceDisableLF"
                  :label="text('Setup.Launch.Advanced.DisableLF')"
                  v-model="values.CheckAdvanceDisableLF"
                />
                <PCheckBox
                  disabled
                  data-control="CheckAdvanceGraphicCard"
                  style="height: 28px"
                  :title="text('Setup.Launch.Advanced.GpuPreference.ToolTip')"
                  id="PageSetupLaunch-CheckAdvanceGraphicCard"
                  :label="text('Setup.Launch.Advanced.GpuPreference')"
                  v-model="values.CheckAdvanceGraphicCard"
                />
                <PCheckBox
                  disabled
                  data-control="CheckAdvanceNoJavaw"
                  style="height: 28px"
                  :title="text('Setup.Launch.Advanced.UseJavaExe.ToolTip')"
                  id="PageSetupLaunch-CheckAdvanceNoJavaw"
                  :label="text('Setup.Launch.Advanced.UseJavaExe')"
                  v-model="values.CheckAdvanceNoJavaw"
                />
                <PCheckBox
                  disabled
                  data-control="CheckAdvanceDisableLwjglUnsafeAgent"
                  style="height: 28px"
                  :title="text('Setup.Launch.Advanced.DisableLwjglUnsafeAgent.ToolTip')"
                  id="PageSetupLaunch-CheckAdvanceDisableLwjglUnsafeAgent"
                  :label="text('Setup.Launch.Advanced.DisableLwjglUnsafeAgent')"
                  v-model="values.CheckAdvanceDisableLwjglUnsafeAgent"
                />
                <PCheckBox
                  disabled
                  data-control="CheckAdvanceDisableCrashAnalysis"
                  style="height: 28px"
                  :title="text('Setup.Launch.Advanced.DisableCrashAnalysis.ToolTip')"
                  id="PageSetupLaunch-CheckAdvanceDisableCrashAnalysis"
                  :label="text('Setup.Launch.Advanced.DisableCrashAnalysis')"
                  v-model="values.CheckAdvanceDisableCrashAnalysis"
                />
                <PCheckBox
                  disabled
                  data-control="CheckAdvanceLockMemory"
                  style="height: 28px"
                  :title="text('Setup.Launch.Advanced.LockMemory.ToolTip')"
                  id="PageSetupLaunch-CheckAdvanceLockMemory"
                  :label="text('Setup.Launch.Advanced.LockMemory')"
                  v-model="values.CheckAdvanceLockMemory"
                />
              </div>
              <PHint
                style="margin: 5px 0px 10px 0px; grid-column: 1 / -1"
                severity="info"
                closable
                >{{ text('Setup.Launch.Advanced.MoreInInstanceSetup') }}</PHint
              >
            </div>
          </PCard>
          <PPrimaryAction
            data-control="BtnSwitch"
            style="margin: 0 0 15px"
            @click="() => router.push('/instance_setting/setting')"
            ><PSvgIcon name="lucide/arrow-right" :size="16" />{{
              text('Setup.Launch.SwitchToInstanceSetup')
            }}</PPrimaryAction
          >
        </div>
      </div>
    </div>
  </div>
</template>
