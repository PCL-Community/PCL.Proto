<script setup lang="ts">
import SetupItem from '@/components/widget/SetupItem.vue'
import { useSetup } from '@/stores/setup'
import { watch, computed, onMounted, onUnmounted, ref } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { useJavaStore } from '@/stores/java'
import { useRouter } from 'vue-router'
import { pclText } from '@/api/pclLocale'
import { usePageSettings } from '@/composables/usePageSettings'
import PPrimaryAction from '@/components/widget/PPrimaryAction.vue'
import PCard from '@/components/widget/PCard.vue'
import PCheckBox from '@/components/widget/PCheckBox.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PHint from '@/components/widget/PHint.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import PTextBox from '@/components/widget/PTextBox.vue'
import { windowSettingKeys, windowSettingsStorage, type WindowSettings } from '@/api/windowSettings'
import { memorySettingKeys } from '@/api/memorySettings'
import PMemorySettings from '@/components/widget/PMemorySettings.vue'
import PErrorRetry from '@/components/widget/PErrorRetry.vue'
import sideTip from '@/composables/sideTip'
import { getIsolationDefault, setIsolationDefault, isolationChoices } from '@/api/isolationSettings'
import { versionInfoStorage } from '@/api/versionInfo'
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
        'TextArgumentInfo',
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
const visibilityError = ref<string | null>(null)
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
    visibilityError.value = null
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
    visibilityError.value = null
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
function resetJvmArguments() {
  argumentJvm.value = sourceDefaults.PageSetupLaunch.TextAdvanceJvm
  saveArguments()
}
const isolationValue = ref('4')
const isolationReady = ref(false)
const isolationBusy = ref(false)
const isolationError = ref<string | null>(null)
const isolationControlRevision = ref(0)
async function loadIsolation() {
  try {
    const value = await getIsolationDefault()
    const index = isolationChoices.indexOf(value)
    if (index < 0) throw new Error('Unknown isolation setting')
    isolationValue.value = String(index)
    isolationReady.value = true
    isolationError.value = null
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
    isolationError.value = null
  } catch (error) {
    isolationError.value = String(error)
  } finally {
    isolationBusy.value = false
    isolationControlRevision.value++
  }
}
if (desktop) void loadIsolation()

const infoReady = ref(false)
const infoError = ref<string | null>(null)
let hydratingInfo = false
async function loadVersionInfo() {
  try {
    const { settings } = await versionInfoStorage.read()
    hydratingInfo = true
    values.TextArgumentInfo = settings
    hydratingInfo = false
    infoReady.value = true
    infoError.value = null
  } catch (error) {
    infoError.value = String(error)
  }
}
function saveVersionInfo() {
  if (!infoReady.value) return
  versionInfoStorage.stage(values.TextArgumentInfo)
  void versionInfoStorage.save().catch((error) => (infoError.value = String(error)))
}
if (desktop) {
  void loadVersionInfo()
  onUnmounted(saveVersionInfo)
}
watch(
  () => values.TextArgumentInfo,
  () => {
    if (!desktop || hydratingInfo) return
    versionInfoStorage.stage(values.TextArgumentInfo)
  },
  { flush: 'sync' },
)
const memoryJava64 = computed(() => (java.runtimes.length ? java.globalIs64Bit : null))

const windowReady = ref(false)
const windowError = ref<string | null>(null)
let hydratingWindow = false
function loadWindowSettings() {
  void windowSettingsStorage
    .read()
    .then(({ settings }) => {
      hydratingWindow = true
      values.ComboArgumentWindowType = { default: '1', launcher: '2', custom: '3' }[settings.mode]
      values.TextArgumentWindowWidth = String(settings.width)
      values.TextArgumentWindowHeight = String(settings.height)
      hydratingWindow = false
      windowReady.value = true
      windowError.value = null
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
  windowError.value = null
  void windowSettingsStorage
    .write({ mode, width, height })
    .catch((error) => sideTip.show(String(error), 'warn'))
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
const unavailableAdvancedOptions = [
  ['CheckAdvanceDisableJLW', 'DisableJlw'],
  ['CheckAdvanceDisableLF', 'DisableLF'],
  ['CheckAdvanceGraphicCard', 'GpuPreference'],
  ['CheckAdvanceNoJavaw', 'UseJavaExe'],
  ['CheckAdvanceDisableLwjglUnsafeAgent', 'DisableLwjglUnsafeAgent'],
  ['CheckAdvanceDisableCrashAnalysis', 'DisableCrashAnalysis'],
  ['CheckAdvanceLockMemory', 'LockMemory'],
] as const
</script>
<template>
  <div class="pcl-page">
    <PErrorRetry
      :error="visibilityError"
      @retry="visibilityReady ? changeVisibility(visibilityDraft) : loadVisibility()"
    />
    <PErrorRetry :error="isolationError" @retry="loadIsolation" />
    <PErrorRetry :error="infoError" @retry="infoReady ? saveVersionInfo() : loadVersionInfo()" />
    <PErrorRetry
      :error="argumentsError"
      @retry="argumentsReady ? saveArguments() : loadArguments()"
    />
    <PErrorRetry
      :error="windowError || windowSettingsStorage.errors.get(null)"
      @retry="windowReady ? applyWindowSettings() : loadWindowSettings()"
    />

    <PCard :title="text('Setup.Launch.Options.Title')">
      <div style="margin: 0px 25px 15px 25px; display: flex; flex-direction: column">
        <div class="settings-grid">
          <SetupItem :label="text('Setup.Launch.Options.InstanceIsolation.Label')" v-slot="{ id }"
            ><PComboBox
              :disabled="!desktop || !isolationReady || isolationBusy"
              :key="isolationControlRevision"
              :input-id="id"
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
              :suggestions="[text('Setup.Launch.Options.WindowTitle.SampleFormat')]"
              :title="text('Setup.Launch.Options.WindowTitle.ToolTip')"
              :placeholder="text('Common.Option.Default')"
              v-model="values.TextArgumentTitle"
          /></SetupItem>
          <SetupItem :label="text('Setup.Launch.Options.CustomInfo.Label')" v-slot="{ id }"
            ><PTextBox
              :disabled="!desktop || !infoReady"
              :input-id="id"
              :title="text('Setup.Launch.Options.CustomInfo.ToolTip')"
              :placeholder="text('')"
              v-model="values.TextArgumentInfo"
          /></SetupItem>
          <SetupItem :label="text('Setup.Launch.Options.Visibility.Label')" v-slot="{ id }"
            ><PComboBox
              :disabled="!desktop || !visibilityReady || visibilityBusy"
              :key="visibilityRevision"
              :input-id="id"
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
            ><div style="display: grid; grid-template-columns: minmax(0, 1fr) auto auto auto">
              <PComboBox
                :disabled="!windowReady"
                :input-id="id"
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
    <PCard :title="text('Setup.Launch.Memory.Title')">
      <PHint v-if="java32Bit" severity="warning">{{
        text('Setup.Launch.Memory.Warning.Java32Bit')
      }}</PHint>
      <PHint v-if="memoryOverLimit" severity="warning">{{
        text('Setup.Launch.Memory.Warning.TooMuch')
      }}</PHint>
      <PMemorySettings :java64="memoryJava64" />
    </PCard>
    <PCard :title="text('Setup.Launch.Advanced.Title')" default-fold-status="fold">
      <div class="settings-grid" style="margin: 0px 25px 15px 25px">
        <SetupItem :label="text('Setup.Launch.Advanced.Renderer.Label')" v-slot="{ id }"
          ><PComboBox
            disabled
            :input-id="id"
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
              <label :for="id" class="pcl-text">{{ text('Setup.Launch.Advanced.JvmHead') }}</label>
              <button
                type="button"
                class="pcl-icon-button"
                :disabled="!desktop || !argumentsReady"
                style="margin: 1px 0px 0px 0px"
                :title="text('Setup.Launch.Advanced.JvmHead.Restore')"
                :aria-label="text('Setup.Launch.Advanced.JvmHead.Restore')"
                @click="resetJvmArguments"
              >
                <PSvgIcon name="lucide/rotate-ccw" :size="18" />
              </button></div></template
          ><template #default="{ id }"
            ><PTextBox
              :disabled="!desktop || !argumentsReady"
              :input-id="id"
              multiline
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
            :title="text('Setup.Launch.Advanced.PreLaunchCommand.ToolTip')"
            :placeholder="text('')"
            v-model="values.TextAdvanceRun"
        /></SetupItem>
        <PCheckBox
          disabled
          v-if="String(values.TextAdvanceRun || '').trim()"
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
            v-for="[key, label] in unavailableAdvancedOptions"
            :key
            disabled
            :title="text(`Setup.Launch.Advanced.${label}.ToolTip`)"
            :id="`PageSetupLaunch-${key}`"
            :label="text(`Setup.Launch.Advanced.${label}`)"
            :model-value="Boolean(values[key])"
          />
        </div>
        <PHint style="margin: 5px 0px 10px 0px; grid-column: 1 / -1" severity="info" closable>{{
          text('Setup.Launch.Advanced.MoreInInstanceSetup')
        }}</PHint>
      </div>
    </PCard>
    <PPrimaryAction @click="() => router.push('/instance_setting/setting')"
      ><PSvgIcon name="lucide/arrow-right" :size="16" />{{
        text('Setup.Launch.SwitchToInstanceSetup')
      }}</PPrimaryAction
    >
  </div>
</template>
