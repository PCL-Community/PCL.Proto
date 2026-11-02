<script setup lang="ts">
import SetupItem from '@/components/widget/SetupItem.vue'
import { useRouter } from 'vue-router'
import { showUnavailable } from '@/util/showUnavailable'
import { pclText } from '@/api/pclLocale'
import { computed, watch, ref } from 'vue'
import { selectRelativeJava, type JavaPreference, type InstanceJavaSettings } from '@/api/javaReq'
import { useJavaStore } from '@/stores/java'
import { useSelectedInstance } from '@/stores/gameLaunch'
import { isTauri } from '@tauri-apps/api/core'
import sideTip from '@/composables/sideTip'
import { usePageSettings } from '@/composables/usePageSettings'
import { memorySettingKeys } from '@/api/memorySettings'
import PMemorySettings from '@/components/widget/PMemorySettings.vue'
import PErrorRetry from '@/components/widget/PErrorRetry.vue'
import PButton from '@/components/widget/PButton.vue'
import PPrimaryAction from '@/components/widget/PPrimaryAction.vue'
import PCard from '@/components/widget/PCard.vue'
import PCheckBox from '@/components/widget/PCheckBox.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PHint from '@/components/widget/PHint.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import PTextBox from '@/components/widget/PTextBox.vue'
import { useModal } from '@/composables/useModal'
import { getInstanceIsolation, setInstanceIsolation } from '@/api/isolationSettings'
import { launchArgumentKeys } from '@/api/launchArguments'
import { useLaunchArguments } from '@/composables/useLaunchArguments'
import { useInstanceLaunchSettings } from '@/composables/useInstanceLaunchSettings'
import currentPlatform, { currentArch } from '@/util/platform'
const props = defineProps<{
  loginLocked?: boolean
  memoryOverLimit?: boolean
  canCreateProfile?: boolean
}>()
const { values } = usePageSettings(
  'PageInstanceSetup',
  {},
  isTauri() ? [...memorySettingKeys, ...launchArgumentKeys, 'ComboArgumentIndieV2'] : [],
)
const java = useJavaStore()
const selected = useSelectedInstance()
const directory = computed(() => selected.instance_info?.directory)
const {
  jvm: argumentJvm,
  game: argumentGame,
  classpath: argumentClasspath,
  ready: argumentsReady,
  error: argumentsError,
  load: loadArguments,
  save: saveArguments,
} = useLaunchArguments(directory)
const memoryJava64 = computed(() =>
  directory.value && java.runtimes.length ? (java.instanceIs64Bit[directory.value] ?? null) : null,
)
const {
  settings: launchSettings,
  ready: launchSettingsReady,
  error: launchSettingsError,
  save: saveLaunchSettings,
} = useInstanceLaunchSettings(directory)
const rendererWindowsOnly = currentPlatform !== 'windows'
const jlwForced = computed(() => currentArch === 'aarch64')

const javaSettings = computed<InstanceJavaSettings>(
  () =>
    (directory.value && java.instances[directory.value]) || {
      preference: { kind: 'global' },
      ignore_compatibility: false,
    },
)
const javaLoading = ref(true)
const javaSaving = ref(false)
const javaChoiceRevision = ref(0)
const javaError = ref(false)
watch(
  directory,
  async (path, _, onCleanup) => {
    let current = true
    onCleanup(() => {
      current = false
    })
    javaLoading.value = true
    javaError.value = false
    try {
      if (!path || !isTauri()) throw new Error('Desktop instance required')
      await java.load(path)
    } catch (err) {
      if (current) javaError.value = true
      console.error(err)
    } finally {
      if (current) javaLoading.value = false
    }
  },
  { immediate: true },
)
const javaChoice = computed(() => {
  const pref = javaSettings.value.preference
  return pref.kind === 'exist' ? pref.java_exe : pref.kind
})
const javaOptions = computed(() => [
  { key: 'global', text: text('Instance.Setup.FollowGlobal') },
  { key: 'auto', text: text('Instance.Setup.Options.Java.AutoSelect') },
  {
    key: 'relative',
    text:
      javaSettings.value.preference.kind === 'relative'
        ? text('Instance.Setup.Options.Java.SelectRelative.WithJava', [
            javaSettings.value.preference.relative_path,
          ])
        : text('Instance.Setup.Options.Java.SelectRelative'),
    title: text('Proto.Java.RelativePathHint'),
  },
  ...java.runtimes.map((runtime) => ({
    key: runtime.java_exe,
    text: `${runtime.version} · ${runtime.architecture} · ${runtime.java_folder}${runtime.is_enabled ? '' : ` · ${text('Setup.Java.EnableBeforeSelect')}`}`,
    disabled: !runtime.is_enabled,
  })),
  ...(javaSettings.value.preference.kind === 'exist' &&
  !java.runtimes.some((j) => j.java_exe === javaChoice.value)
    ? [
        {
          key: javaChoice.value,
          text: `${javaChoice.value} · ${text('Setup.Java.Unavailable')}`,
          disabled: true,
        },
      ]
    : []),
])
async function saveJava(settings: InstanceJavaSettings) {
  const path = directory.value
  if (!path || javaSaving.value || javaLoading.value) return
  javaSaving.value = true
  try {
    await java.saveInstance(path, settings)
  } catch (err) {
    sideTip.show(String(err), 'warn')
  } finally {
    javaSaving.value = false
  }
}
async function selectJava(choice: string | undefined) {
  const path = directory.value
  if (!path || !choice) return
  let preference: JavaPreference
  try {
    if (choice === 'relative') {
      const picked = await selectRelativeJava()
      if (!picked || path !== directory.value) return
      preference = picked
    } else if (choice === 'global' || choice === 'auto') preference = { kind: choice }
    else preference = { kind: 'exist', java_exe: choice }
    await saveJava({ ...javaSettings.value, preference })
  } catch (err) {
    sideTip.show(String(err), 'warn')
  } finally {
    javaChoiceRevision.value++
  }
}
const java32Bit = computed(
  () =>
    !!directory.value &&
    java.runtimes.length > 0 &&
    java.instanceIs64Bit[directory.value] === false,
)
const authVisible = computed(() => ['2', '3'].includes(String(values.ComboServerLoginRequire)))
const modal = useModal()
// CE remembers these confirmations for the whole run; the page only warns once.
let rendererHinted = false
let debugLogHinted = false
async function confirmWarning(message: string) {
  const result = await modal.open({
    title: text('Common.Dialog.Warning'),
    content: message,
    buttons: [
      {
        content: text('Setup.Launch.Advanced.Renderer.Warning.Confirm'),
        operation: () => modal.close(true),
      },
      { content: text('Common.Action.Cancel'), operation: () => modal.close(false) },
    ],
  })
  return !!result.confirmed
}
async function changeRenderer(value: string | undefined) {
  if (!value || Number(value) === launchSettings.renderer) return
  if (value !== '0' && !rendererHinted) {
    if (!(await confirmWarning(text('Setup.Launch.Advanced.Renderer.Warning.Message')))) return
    rendererHinted = true
  }
  launchSettings.renderer = Number(value)
  saveLaunchSettings()
}
async function changeDebugLog(value: boolean) {
  if (value === launchSettings.debug_log4j2) return
  if (value && !debugLogHinted) {
    if (!(await confirmWarning(text('Instance.Setup.Advanced.UseDebugLog4j.Message')))) return
    debugLogHinted = true
  }
  launchSettings.debug_log4j2 = value
  saveLaunchSettings()
}
const isolationValue = ref('0')
const isolationReady = ref(false)
const isolationBusy = ref(false)
const isolationError = ref('')
const isolationControlRevision = ref(0)
let isolationRevision = 0
async function loadIsolation(path = directory.value) {
  const revision = ++isolationRevision
  isolationReady.value = false
  isolationError.value = ''
  if (!path || !isTauri()) return
  try {
    const enabled = await getInstanceIsolation(path)
    if (revision !== isolationRevision) return
    isolationValue.value = enabled ? '0' : '1'
    isolationReady.value = true
  } catch (error) {
    if (revision === isolationRevision) isolationError.value = String(error)
  }
}
watch(
  directory,
  (path) => {
    void loadIsolation(path)
  },
  { immediate: true },
)
async function changeIsolation(value: string | undefined) {
  const path = directory.value
  if (
    !path ||
    !isolationReady.value ||
    isolationBusy.value ||
    (value !== '0' && value !== '1') ||
    value === isolationValue.value
  )
    return
  isolationBusy.value = true
  const revision = isolationRevision
  try {
    const result = await modal.open({
      title: text('Common.Dialog.Warning'),
      content: text('Instance.Setup.Options.InstanceIsolation.Message'),
      buttons: [
        {
          content: text('Setup.Launch.Advanced.Renderer.Warning.Confirm'),
          operation: () => modal.close(true),
        },
        { content: text('Common.Action.Cancel'), operation: () => modal.close(false) },
      ],
    })
    if (!result.confirmed || revision !== isolationRevision || path !== directory.value) return
    await setInstanceIsolation(path, value === '0')
    if (revision === isolationRevision) {
      isolationValue.value = value
      isolationError.value = ''
    }
  } catch (error) {
    if (revision === isolationRevision) isolationError.value = String(error)
  } finally {
    isolationBusy.value = false
    isolationControlRevision.value++
  }
}
async function setLittleSkin() {
  if (props.loginLocked) return
  const server = 'https://littleskin.cn/api/yggdrasil'
  if (values.TextServerAuthServer && values.TextServerAuthServer !== server) {
    const result = await modal.open({
      title: text('Instance.Setup.Server.LittleSkin.Override.Title'),
      content: text('Instance.Setup.Server.LittleSkin.Override.Message'),
      buttons: [
        {
          content: text('Instance.Setup.Server.LittleSkin.Override.Continue'),
          operation: () => modal.close(true),
        },
        { content: text('Common.Action.Cancel'), operation: () => modal.close(false) },
      ],
    })
    if (!result.confirmed) return
  }
  values.TextServerAuthServer = server
  values.TextServerAuthRegister = 'https://littleskin.cn/auth/register'
  values.TextServerAuthName = text('Instance.Setup.Server.LittleSkin.Name')
}

const router = useRouter()
const text = pclText
</script>
<template>
  <div class="pcl-page">
    <p class="settings-unavailable">{{ text('Proto.Settings.Unavailable') }}</p>
    <PErrorRetry :error="isolationError" @retry="loadIsolation()" />
    <PErrorRetry
      :error="argumentsError"
      @retry="argumentsReady ? saveArguments() : loadArguments()"
    />
    <PErrorRetry :error="launchSettingsError" @retry="saveLaunchSettings()" />
    <div>
      <div data-control="PanBack">
        <div
          data-control="PanMain"
          style="margin: 25px 25px 25px 25px; display: flex; flex-direction: column"
        >
          <PHint style="margin: 0px 0px 15px 0px" severity="info" closable>{{
            text('Instance.Setup.Hint')
          }}</PHint>
          <PCard
            data-control="CardArgument"
            style="margin: 0px 0px 15px 0px"
            :title="text('Setup.Launch.Options.Title')"
          >
            <div style="margin: 0px 25px 21px 25px; display: flex; flex-direction: column">
              <div class="settings-grid">
                <SetupItem :label="text('Instance.Setup.Options.InstanceIsolation')" v-slot="{ id }"
                  ><PComboBox
                    :disabled="!isolationReady || isolationBusy"
                    :key="isolationControlRevision"
                    :input-id="id"
                    data-control="ComboArgumentIndieV2"
                    :options="[
                      {
                        key: '0',
                        text: text('Instance.Setup.Options.InstanceIsolation.Enabled'),
                        title: text('Instance.Setup.Options.InstanceIsolation.Enabled.ToolTip'),
                      },
                      {
                        key: '1',
                        text: text('Common.Action.Close'),
                        title: text('Instance.Setup.Options.InstanceIsolation.Disabled.ToolTip'),
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
                    :title="text('Instance.Setup.Options.WindowTitle.ToolTip')"
                    :placeholder="text('')"
                    v-model="values.TextArgumentTitle"
                /></SetupItem>
                <PCheckBox
                  disabled
                  data-control="CheckArgumentTitleEmpty"
                  style="margin: 0px 0px 9px 0px; grid-column: 1 / -1"
                  :title="text('Instance.Setup.Options.WindowTitle.Default.ToolTip')"
                  id="PageInstanceSetup-CheckArgumentTitleEmpty"
                  :label="text('Instance.Setup.Options.WindowTitle.Default')"
                  v-model="values.CheckArgumentTitleEmpty"
                />
                <SetupItem :label="text('Setup.Launch.Options.CustomInfo.Label')" v-slot="{ id }"
                  ><PTextBox
                    :disabled="!launchSettingsReady"
                    :input-id="id"
                    data-control="TextArgumentInfo"
                    :title="text('Instance.Setup.Options.CustomInfo.ToolTip')"
                    :placeholder="text('Instance.Setup.FollowGlobal')"
                    v-model="launchSettings.info"
                    @blur="saveLaunchSettings()"
                /></SetupItem>
                <SetupItem :label="text('Instance.Setup.Options.Java')" v-slot="{ id }"
                  ><PComboBox
                    :key="javaChoiceRevision"
                    :input-id="id"
                    data-control="ComboArgumentJava"
                    :title="text('Instance.Setup.Options.Java.ToolTip')"
                    :options="
                      javaLoading
                        ? [{ key: '0', text: text('Instance.Setup.Options.Java.Loading') }]
                        : javaOptions
                    "
                    :disabled="javaLoading || javaSaving || javaError"
                    :model-value="javaChoice"
                    @update:model-value="selectJava"
                /></SetupItem>
              </div>
            </div>
          </PCard>
          <PHint v-if="javaError" severity="warning" style="margin-bottom: 15px"
            >{{ text('Instance.Setup.Options.Java.LoadFailed')
            }}<RouterLink to="/setup/java">{{
              text('Instance.Setup.Options.Java')
            }}</RouterLink></PHint
          >
          <PCard style="margin: 0px 0px 15px 0px" :title="text('Setup.Launch.Memory.Title')">
            <PHint v-if="java32Bit" severity="warning">{{
              text('Setup.Launch.Memory.Warning.Java32Bit')
            }}</PHint>
            <PHint v-if="memoryOverLimit" severity="warning">{{
              text('Setup.Launch.Memory.Warning.TooMuch')
            }}</PHint>
            <PMemorySettings instance :directory="directory" :java64="memoryJava64" />
          </PCard>
          <PCard
            data-control="CardServer"
            style="margin: 0px 0px 15px 0px"
            :title="text('Instance.Setup.Server.Title')"
          >
            <div style="margin: 2px 25px 18px 25px; display: flex; flex-direction: column">
              <PHint v-if="loginLocked" severity="warning" style="margin-bottom: 13px">{{
                text('Instance.Setup.Server.LoginLockHint')
              }}</PHint>
              <PHint
                v-if="authVisible && /^https:\/\//i.test(String(values.TextServerAuthServer))"
                severity="info"
                style="margin-bottom: 13px"
                >{{ text('Instance.Setup.Server.SslWarning') }}</PHint
              >
              <PHint
                v-if="authVisible && /^http:\/\//i.test(String(values.TextServerAuthServer))"
                severity="warning"
                style="margin-bottom: 13px"
                >{{ text('Instance.Setup.Server.HttpWarning') }}</PHint
              >
              <div
                style="
                  display: grid;
                  grid-template-columns: auto minmax(0, 1fr);
                  grid-template-rows: 28px auto auto auto 9px 28px 3px;
                "
              >
                <span
                  style="
                    margin: 0px 25px 0px 0px;
                    grid-row: 1 / span 1;
                    grid-column: 1 / span 1;
                    justify-self: start;
                    align-self: center;
                  "
                  class="pcl-text"
                  >{{ text('Instance.Setup.Server.RestrictLogin') }}</span
                >
                <PComboBox
                  data-control="ComboServerLoginRequire"
                  disabled
                  style="grid-row: 1 / span 1; grid-column: 2 / span 2"
                  :options="[
                    { key: '0', text: text('Instance.Setup.Server.LoginMethod.Unlimited') },
                    { key: '1', text: text('Instance.Setup.Server.LoginMethod.MicrosoftOnly') },
                    { key: '2', text: text('Instance.Setup.Server.LoginMethod.AuthOnly') },
                    { key: '3', text: text('Instance.Setup.Server.LoginMethod.MicrosoftOrAuth') },
                  ]"
                  v-model="values.ComboServerLoginRequire"
                />
                <template v-if="authVisible">
                  <label
                    for="auth-Server"
                    style="grid-row: 2; align-self: center; margin: 8px 25px 0 0"
                    >{{ text('Instance.Setup.Server.AuthServer') }}</label
                  >
                  <PTextBox
                    data-control="TextServerAuthServer"
                    disabled
                    input-id="auth-Server"
                    style="grid-row: 2; grid-column: 2 / span 2; margin-top: 8px"
                    :placeholder="text('Instance.Setup.Server.AuthServer.Hint')"
                    v-model="values.TextServerAuthServer as string"
                  />
                </template>
                <template v-if="authVisible">
                  <label
                    for="auth-Register"
                    style="grid-row: 3; align-self: center; margin: 8px 25px 0 0"
                    >{{ text('Instance.Setup.Server.Register') }}</label
                  >
                  <PTextBox
                    data-control="TextServerAuthRegister"
                    disabled
                    input-id="auth-Register"
                    style="grid-row: 3; grid-column: 2 / span 2; margin-top: 8px"
                    :placeholder="text('Instance.Setup.Server.Register.Hint')"
                    v-model="values.TextServerAuthRegister as string"
                  />
                </template>
                <template v-if="authVisible">
                  <label
                    for="auth-Name"
                    style="grid-row: 4; align-self: center; margin: 8px 25px 0 0"
                    >{{ text('Instance.Setup.Server.Name') }}</label
                  >
                  <PTextBox
                    data-control="TextServerAuthName"
                    disabled
                    input-id="auth-Name"
                    style="grid-row: 4; grid-column: 2 / span 2; margin-top: 8px"
                    :placeholder="text('Instance.Setup.Server.Name.Hint')"
                    v-model="values.TextServerAuthName as string"
                  />
                </template>
                <span
                  style="
                    margin: 0px 25px 0px 0px;
                    grid-row: 6 / span 1;
                    grid-column: 1 / span 1;
                    justify-self: start;
                    align-self: center;
                  "
                  class="pcl-text"
                  >{{ text('Instance.Setup.Server.AutoJoin') }}</span
                >
                <PTextBox
                  disabled
                  data-control="TextServerEnter"
                  style="grid-row: 6 / span 1; grid-column: 2 / span 2"
                  :title="text('Instance.Setup.Server.AutoJoin.ToolTip')"
                  :placeholder="text('')"
                  v-model="values.TextServerEnter"
                />
              </div>
              <div
                v-if="authVisible"
                style="display: flex; gap: 30px; flex-wrap: wrap; margin-top: 12px"
              >
                <PButton
                  data-control="BtnServerAuthLittle"
                  style="height: 35px; min-width: 150px; margin: 0"
                  @click="setLittleSkin"
                  disabled
                  type="tint"
                  >{{ text('Instance.Setup.Server.SetLittleSkin') }}</PButton
                >
                <PButton
                  data-control="BtnServerAuthLock"
                  style="height: 35px; min-width: 150px; margin: 0"
                  @click="showUnavailable"
                  disabled
                  >{{ text('Instance.Setup.Server.LockLoginMethod') }}</PButton
                >
                <PButton
                  v-if="canCreateProfile"
                  data-control="BtnServerNewProfile"
                  style="height: 35px; min-width: 150px; margin: 0"
                  @click="showUnavailable"
                  >{{ text('Instance.Setup.Server.CreateProfile') }}</PButton
                >
              </div>
            </div>
          </PCard>
          <PCard
            data-control="CardAdvance"
            style="margin: 0px 0px 15px 0px"
            :title="text('Setup.Launch.Advanced.Title')"
            default-fold-status="fold"
          >
            <div class="settings-grid" style="margin: 0px 25px 15px 25px">
              <SetupItem :label="text('Setup.Launch.Advanced.Renderer.Label')" v-slot="{ id }"
                ><PComboBox
                  :disabled="!launchSettingsReady || rendererWindowsOnly"
                  :input-id="id"
                  data-control="ComboAdvanceRenderer"
                  :title="
                    rendererWindowsOnly ? text('Proto.Settings.RendererWindowsOnly') : undefined
                  "
                  :options="[
                    { key: '0', text: text('Instance.Setup.FollowGlobal') },
                    { key: '1', text: text('Setup.Launch.Advanced.Renderer.GameDefault') },
                    { key: '2', text: text('Setup.Launch.Advanced.Renderer.Software') },
                    { key: '3', text: text('Setup.Launch.Advanced.Renderer.DirectX12') },
                    { key: '4', text: text('Setup.Launch.Advanced.Renderer.Vulkan') },
                  ]"
                  :model-value="String(launchSettings.renderer)"
                  @update:model-value="changeRenderer"
              /></SetupItem>
              <SetupItem :label="text('Setup.Launch.Advanced.JvmHead')" v-slot="{ id }"
                ><PTextBox
                  :disabled="!argumentsReady"
                  :input-id="id"
                  multiline
                  data-control="TextAdvanceJvm"
                  style="height: 100px; padding: 5px 0px"
                  :title="
                    text('Instance.Setup.Advanced.JvmArgs.ToolTip') +
                    '\n\n' +
                    text('Proto.LaunchArguments.Parsing') +
                    '\n' +
                    text('Proto.LaunchArguments.Hint')
                  "
                  :placeholder="text('Instance.Setup.FollowGlobal')"
                  v-model="argumentJvm"
                  @blur="saveArguments()"
              /></SetupItem>
              <SetupItem :label="text('Setup.Launch.Advanced.GameTail')" v-slot="{ id }"
                ><PTextBox
                  :disabled="!argumentsReady"
                  :input-id="id"
                  data-control="TextAdvanceGame"
                  :title="
                    text('Instance.Setup.Advanced.GameArgs.ToolTip') +
                    '\n\n' +
                    text('Proto.LaunchArguments.Parsing')
                  "
                  :placeholder="text('Instance.Setup.FollowGlobal')"
                  v-model="argumentGame"
                  @blur="saveArguments()"
              /></SetupItem>
              <SetupItem :label="text('Instance.Setup.Advanced.Classpath')" v-slot="{ id }"
                ><PTextBox
                  :disabled="!argumentsReady"
                  :input-id="id"
                  data-control="TextAdvanceClasspathHead"
                  :title="text('Instance.Setup.Advanced.Classpath.ToolTip')"
                  :placeholder="text('')"
                  v-model="argumentClasspath"
                  @blur="saveArguments()"
              /></SetupItem>
              <SetupItem :label="text('Setup.Launch.Advanced.PreLaunchCommand')" v-slot="{ id }"
                ><PTextBox
                  :disabled="!launchSettingsReady"
                  :input-id="id"
                  data-control="TextAdvanceRun"
                  style="margin: 0px 0px 3px 0px"
                  :title="text('Instance.Setup.Advanced.PreLaunchCommand.ToolTip')"
                  :placeholder="text('')"
                  v-model="launchSettings.prelaunch_command"
                  @blur="saveLaunchSettings()"
              /></SetupItem>
              <div
                style="
                  margin: 8px 0px 4px 0px;
                  display: flex;
                  flex-direction: column;
                  grid-column: 1 / -1;
                "
              >
                <PCheckBox
                  v-if="launchSettings.prelaunch_command.trim()"
                  data-control="CheckAdvanceRunWait"
                  style="margin: 0 0 8px"
                  id="PageInstanceSetup-CheckAdvanceRunWait"
                  :disabled="!launchSettingsReady"
                  :label="text('Setup.Launch.Advanced.PreLaunchCommand.Wait')"
                  :model-value="launchSettings.prelaunch_wait"
                  @update:model-value="
                    (value) => {
                      launchSettings.prelaunch_wait = value
                      saveLaunchSettings()
                    }
                  "
                />

                <PCheckBox
                  disabled
                  data-control="CheckAdvanceDisableModUpdate"
                  style="height: 28px"
                  :title="text('Proto.Settings.ModUpdateUnavailable')"
                  id="PageInstanceSetup-CheckAdvanceDisableModUpdate"
                  :label="text('Instance.Setup.Advanced.DisableModUpdate')"
                  v-model="values.CheckAdvanceDisableModUpdate"
                />
                <PCheckBox
                  data-control="CheckAdvanceJava"
                  style="height: 28px"
                  :title="text('Instance.Setup.Advanced.IgnoreJavaCompatibility.ToolTip')"
                  id="PageInstanceSetup-CheckAdvanceJava"
                  :label="text('Instance.Setup.Advanced.IgnoreJavaCompatibility')"
                  :disabled="javaLoading || javaSaving || javaError"
                  :model-value="javaSettings.ignore_compatibility"
                  @update:model-value="
                    (value) => saveJava({ ...javaSettings, ignore_compatibility: value })
                  "
                />
                <PCheckBox
                  data-control="CheckAdvanceAssetsV2"
                  style="height: 28px; margin: 0px 60px 0px 0px"
                  :title="text('Instance.Setup.Advanced.DisableFileVerification.ToolTip')"
                  id="PageInstanceSetup-CheckAdvanceAssetsV2"
                  :disabled="!launchSettingsReady"
                  :label="text('Instance.Setup.Advanced.DisableFileVerification')"
                  :model-value="launchSettings.disable_asset_verify"
                  @update:model-value="
                    (value) => {
                      launchSettings.disable_asset_verify = value
                      saveLaunchSettings()
                    }
                  "
                />
                <PCheckBox
                  data-control="CheckAdvanceUseProxyV2"
                  style="height: 28px"
                  :title="text('Instance.Setup.Advanced.UseGlobalProxy.ToolTip')"
                  id="PageInstanceSetup-CheckAdvanceUseProxyV2"
                  :disabled="!launchSettingsReady"
                  :label="text('Instance.Setup.Advanced.UseGlobalProxy')"
                  :model-value="launchSettings.use_proxy"
                  @update:model-value="
                    (value) => {
                      launchSettings.use_proxy = value
                      saveLaunchSettings()
                    }
                  "
                />
                <PCheckBox
                  data-control="CheckAdvanceDisableJLW"
                  style="height: 28px"
                  :title="
                    jlwForced
                      ? text('Setup.Launch.Advanced.DisableJlw.Arm64ToolTip')
                      : text('Setup.Launch.Advanced.DisableJlw.ToolTip')
                  "
                  id="PageInstanceSetup-CheckAdvanceDisableJLW"
                  :disabled="jlwForced || !launchSettingsReady"
                  :label="text('Setup.Launch.Advanced.DisableJlw')"
                  :model-value="jlwForced || launchSettings.disable_jlw"
                  @update:model-value="
                    (value) => {
                      launchSettings.disable_jlw = value
                      saveLaunchSettings()
                    }
                  "
                />
                <PCheckBox
                  data-control="CheckAdvanceDisableLF"
                  style="height: 28px"
                  :title="text('Setup.Launch.Advanced.DisableLF.ToolTip')"
                  id="PageInstanceSetup-CheckAdvanceDisableLF"
                  :disabled="!launchSettingsReady"
                  :label="text('Setup.Launch.Advanced.DisableLF')"
                  :model-value="launchSettings.disable_legacy_fix"
                  @update:model-value="
                    (value) => {
                      launchSettings.disable_legacy_fix = value
                      saveLaunchSettings()
                    }
                  "
                />
                <PCheckBox
                  data-control="CheckUseDebugLog4j2Config"
                  style="height: 28px"
                  id="PageInstanceSetup-CheckUseDebugLog4j2Config"
                  :disabled="!launchSettingsReady"
                  :label="text('Instance.Setup.Advanced.UseDebugLog4j')"
                  :model-value="launchSettings.debug_log4j2"
                  @update:model-value="changeDebugLog"
                />
                <PCheckBox
                  data-control="CheckAdvanceDisableLwjglUnsafeAgent"
                  style="height: 28px"
                  :title="text('Setup.Launch.Advanced.DisableLwjglUnsafeAgent.ToolTip')"
                  id="PageInstanceSetup-CheckAdvanceDisableLwjglUnsafeAgent"
                  :disabled="!launchSettingsReady"
                  :label="text('Setup.Launch.Advanced.DisableLwjglUnsafeAgent')"
                  :model-value="launchSettings.disable_lwjgl_unsafe_agent"
                  @update:model-value="
                    (value) => {
                      launchSettings.disable_lwjgl_unsafe_agent = value
                      saveLaunchSettings()
                    }
                  "
                />
              </div>
            </div>
          </PCard>
          <PPrimaryAction data-control="BtnSwitch" @click="() => router.push('/setup/launch')"
            ><PSvgIcon name="lucide/arrow-right" :size="16" />{{
              text('Instance.Setup.OpenGlobalSettings')
            }}</PPrimaryAction
          >
        </div>
      </div>
    </div>
  </div>
</template>
