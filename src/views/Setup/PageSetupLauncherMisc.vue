<script setup lang="ts">
import { showUnavailable } from '@/util/showUnavailable'
import { pclText } from '@/api/pclLocale'
import { usePageSettings } from '@/composables/usePageSettings'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PCheckBox from '@/components/widget/PCheckBox.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PHint from '@/components/widget/PHint.vue'
import PTextBox from '@/components/widget/PTextBox.vue'
import { logSlider, logSaveError, saveLogSettings, updateLogSlider } from '@/api/logSettings'
import { useProxySettings } from '@/composables/useProxySettings'
import sideTip from '@/composables/sideTip'
const { values } = usePageSettings(
  'PageSetupLauncherMisc',
  {
    ComboSystemActivity: '0',
  },
  ['SliderMaxLog'],
)
const {
  mode: proxyMode,
  address: proxyAddress,
  ready: proxyReady,
  busy: proxyBusy,
  error: proxyError,
  save: saveProxy,
} = useProxySettings()
async function applyProxy() {
  await saveProxy()
  if (!proxyError.value) sideTip.show(text('Proto.Settings.ProxySaved'), 'success')
}

const text = pclText
</script>
<template>
  <div class="pcl-page">
    <p class="settings-unavailable">{{ text('Proto.Settings.Unavailable') }}</p>
    <p v-if="logSaveError" role="alert">
      {{ logSaveError }}
      <button @click="saveLogSettings">{{ text('Launch.Homepage.LoadFailed.Retry') }}</button>
    </p>
    <p v-if="proxyError" role="alert">
      {{ proxyError }}
      <button @click="saveProxy()">{{ text('Launch.Homepage.LoadFailed.Retry') }}</button>
    </p>
    <div>
      <div data-control="PanBack">
        <div
          data-control="PanMain"
          style="margin: 10px 25px; display: flex; flex-direction: column"
        >
          <PCard style="margin: 15px 0px" :title="text('Setup.Misc.System.Title')">
            <div
              style="
                margin: 0px 25px 16px 25px;
                display: grid;
                grid-template-columns: auto minmax(0, 1fr);
                grid-template-rows: 28px 9px 28px 5px 28px 5px 20px 8px auto;
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
                >{{ text('Setup.Misc.System.Announcement') }}</span
              >
              <PComboBox
                disabled
                data-control="ComboSystemActivity"
                style="grid-row: 1 / span 1; grid-column: 2 / span 1"
                :options="[
                  {
                    key: '0',
                    text: text('Setup.Misc.System.Announcement.All'),
                    title: text('Setup.Misc.System.Announcement.All.ToolTip'),
                  },
                  { key: '1', text: text('Setup.Misc.System.Announcement.ImportantOnly') },
                  { key: '2', text: text('Setup.Misc.System.Announcement.Disabled') },
                ]"
                v-model="values.ComboSystemActivity"
              />
              <span
                style="
                  margin: 0px 25px 0px 0px;
                  grid-row: 3 / span 1;
                  grid-column: 1 / span 1;
                  justify-self: start;
                  align-self: center;
                "
                class="pcl-text"
                >{{ text('Setup.Misc.System.MaxFps') }}</span
              >
              <input
                disabled
                data-control="SliderAniFPS"
                style="grid-row: 3 / span 1; grid-column: 2 / span 1"
                :title="text('Setup.Misc.System.MaxFps.ToolTip')"
                type="range"
                min="0"
                max="59"
                v-model.number="values['SliderAniFPS']"
              />
              <span
                style="
                  margin: 0px 25px 0px 0px;
                  grid-row: 5 / span 1;
                  grid-column: 1 / span 1;
                  justify-self: start;
                  align-self: center;
                "
                class="pcl-text"
                >{{ text('Setup.Misc.System.MaxLogLines') }}</span
              >
              <input
                data-control="SliderMaxLog"
                style="grid-row: 5 / span 1; grid-column: 2 / span 1"
                :title="text('Setup.Misc.System.MaxLogLines.ToolTip')"
                type="range"
                min="0"
                max="29"
                :value="logSlider"
                @input="updateLogSlider(Number(($event.target as HTMLInputElement).value))"
              />
              <div
                style="
                  grid-row: 7 / span 1;
                  grid-column: 1 / span 5;
                  display: grid;
                  grid-template-columns: auto auto auto auto;
                  grid-template-rows: 20px;
                "
              >
                <PCheckBox
                  disabled
                  data-control="CheckSystemDisableHardwareAcceleration"
                  style="grid-row: 1 / span 1; grid-column: 1 / span 1"
                  id="PageSetupLauncherMisc-CheckSystemDisableHardwareAcceleration"
                  :label="text('Setup.Misc.System.DisableHardwareAcceleration')"
                  v-model="values.CheckSystemDisableHardwareAcceleration"
                />
                <PCheckBox
                  disabled
                  data-control="CheckSystemTelemetry"
                  style="grid-row: 1 / span 1; grid-column: 2 / span 1"
                  :title="text('Setup.Misc.System.Telemetry.ToolTip')"
                  id="PageSetupLauncherMisc-CheckSystemTelemetry"
                  :label="text('Setup.Misc.System.Telemetry')"
                  v-model="values.CheckSystemTelemetry"
                />
              </div>
              <div
                style="
                  grid-row: 9 / span 1;
                  grid-column: 1 / span 5;
                  display: grid;
                  grid-template-columns: auto auto auto auto;
                  grid-template-rows: 35px;
                "
              >
                <PButton
                  data-control="BtnSystemSettingExp"
                  style="
                    min-width: 140px;
                    margin: 0px 20px 0px 0px;
                    padding: 0px 13px;
                    grid-row: 1 / span 1;
                    grid-column: 1 / span 1;
                  "
                  @click="showUnavailable"
                  >{{ text('Setup.Misc.System.ExportSettings') }}</PButton
                >
                <PButton
                  data-control="BtnSystemSettingImp"
                  style="
                    min-width: 140px;
                    margin: 0px 20px 0px 0px;
                    padding: 0px 13px;
                    grid-row: 1 / span 1;
                    grid-column: 2 / span 1;
                  "
                  @click="showUnavailable"
                  >{{ text('Setup.Misc.System.ImportSettings') }}</PButton
                >
                <PButton
                  data-control="BtnSystemStopUsingPclCe"
                  style="
                    min-width: 140px;
                    margin: 0px 20px 0px 0px;
                    padding: 0px 13px;
                    grid-row: 1 / span 1;
                    grid-column: 3 / span 1;
                  "
                  @click="() => showUnavailable()"
                  type="warn"
                  >{{ text('Setup.Misc.System.StopUsingPclCe') }}</PButton
                >
              </div>
            </div>
          </PCard>
          <PCard style="margin: 0px 0px 15px 0px" :title="text('Setup.Misc.Network.Title')">
            <div
              style="
                margin: 0px 25px 15px 25px;
                display: grid;
                grid-template-columns: auto minmax(0, 1fr);
                grid-template-rows: 28px 4px 28px 4px auto;
              "
            >
              <PCheckBox
                disabled
                data-control="CheckNetDohEnable"
                style="grid-row: 1 / span 1; grid-column: 1 / span 2"
                :title="text('Setup.Misc.Network.DoH.ToolTip')"
                id="PageSetupLauncherMisc-CheckNetDohEnable"
                :label="text('Setup.Misc.Network.DoH')"
                v-model="values.CheckNetDohEnable"
              />
              <span
                style="
                  margin: 0px 25px 0px 0px;
                  grid-row: 3 / span 1;
                  grid-column: 1 / span 1;
                  justify-self: start;
                  align-self: center;
                "
                class="pcl-text"
                >{{ text('Setup.Misc.Network.HttpProxy') }}</span
              >
              <div
                style="
                  grid-row: 3 / span 1;
                  grid-column: 2 / span 1;
                  display: grid;
                  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) minmax(0, 1fr) minmax(
                      0,
                      0.2fr
                    );
                "
              >
                <label
                  data-control="RadioHttpProxyType0"
                  style="height: 22px; grid-row: 1 / span 1; grid-column: 1 / span 1"
                  class="pcl-radio"
                  ><input :disabled="!proxyReady" type="radio" v-model="proxyMode" value="off" />{{
                    text('Setup.Misc.Network.Proxy.None')
                  }}</label
                >
                <label
                  data-control="RadioHttpProxyType1"
                  style="height: 22px; grid-row: 1 / span 1; grid-column: 2 / span 1"
                  class="pcl-radio"
                  ><input
                    :disabled="!proxyReady"
                    type="radio"
                    v-model="proxyMode"
                    value="system"
                  />{{ text('Setup.Misc.Network.Proxy.System') }}</label
                >
                <label
                  data-control="RadioHttpProxyType2"
                  style="height: 22px; grid-row: 1 / span 1; grid-column: 3 / span 1"
                  class="pcl-radio"
                  ><input
                    :disabled="!proxyReady"
                    type="radio"
                    v-model="proxyMode"
                    value="custom"
                  />{{ text('Setup.Misc.Network.Proxy.Custom') }}</label
                >
              </div>
              <div
                data-control="HttpProxyCustom"
                style="
                  grid-row: 5 / span 1;
                  grid-column: 2 / span 1;
                  display: grid;
                  grid-template-columns: auto minmax(0, 1fr) auto minmax(0, 1fr) auto;
                  grid-template-rows: 28px 9px 28px 9px 28px;
                "
              >
                <PHint style="grid-row: 1 / span 1; grid-column: 1 / span 5" severity="warning">{{
                  text('Setup.Misc.Network.Proxy.UntrustedWarning')
                }}</PHint>
                <PTextBox
                  data-control="TextSystemHttpProxy"
                  style="grid-row: 3 / span 1; grid-column: 1 / span 5"
                  :disabled="!proxyReady || proxyMode !== 'custom'"
                  :title="text('Setup.Misc.Network.Proxy.InvalidWarning')"
                  :placeholder="text('Proto.Settings.ProxyExample')"
                  v-model="proxyAddress"
                />
                <span
                  style="
                    margin: 0px 5px 0px 0px;
                    grid-row: 5 / span 1;
                    grid-column: 1 / span 1;
                    justify-self: start;
                    align-self: center;
                  "
                  class="pcl-text"
                  >{{ text('Setup.Misc.Network.Proxy.Username') }}</span
                >
                <PTextBox
                  disabled
                  data-control="TextSystemHttpProxyCustomUsername"
                  style="margin: 0px 5px 0px 0px; grid-row: 5 / span 1; grid-column: 2 / span 1"
                  :title="text('Proto.Settings.ProxyCredentialsHint')"
                  :placeholder="text('Setup.Misc.Network.Proxy.Optional')"
                  v-model="values.TextSystemHttpProxyCustomUsername"
                />
                <span
                  style="
                    margin: 0px 5px 0px 0px;
                    grid-row: 5 / span 1;
                    grid-column: 3 / span 1;
                    justify-self: start;
                    align-self: center;
                  "
                  class="pcl-text"
                  >{{ text('Setup.Misc.Network.Proxy.Password') }}</span
                >
                <PTextBox
                  disabled
                  type="password"
                  data-control="TextSystemHttpProxyCustomPassword"
                  style="margin: 0px 5px 0px 0px; grid-row: 5 / span 1; grid-column: 4 / span 1"
                  :title="text('Proto.Settings.ProxyCredentialsHint')"
                  :placeholder="text('Setup.Misc.Network.Proxy.Optional')"
                  v-model="values.TextSystemHttpProxyCustomPassword"
                />
                <PButton
                  data-control="BtnApplyHttpProxy"
                  style="grid-row: 5 / span 1; grid-column: 5 / span 1"
                  :disabled="!proxyReady || proxyBusy"
                  @click="applyProxy"
                  >{{ text('Setup.Misc.Network.Proxy.Apply') }}</PButton
                >
              </div>
            </div>
          </PCard>
          <PCard
            data-control="CardDebug"
            style="margin: 0px 0px 15px 0px"
            :title="text('Setup.Misc.Debug.Title')"
            default-fold-status="fold"
          >
            <div style="margin: 0px 25px 15px 25px; display: flex; flex-direction: column">
              <div
                data-control="PanDebugAnim"
                style="
                  height: 22px;
                  margin: 0px 0px 10px 0px;
                  display: grid;
                  grid-template-columns: auto minmax(0, 1fr);
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
                  >{{ text('Setup.Misc.Debug.AnimationSpeed') }}</span
                >
                <input
                  disabled
                  data-control="SliderDebugAnim"
                  style="grid-row: 1 / span 1; grid-column: 2 / span 1"
                  type="range"
                  min="0"
                  max="30"
                  v-model.number="values['SliderDebugAnim']"
                />
              </div>
              <div
                style="
                  display: grid;
                  grid-template-columns: minmax(0, 1.5fr) minmax(0, 1fr) minmax(0, 1fr);
                "
              >
                <PCheckBox
                  disabled
                  data-control="CheckDebugSkipCopy"
                  style="height: 22px; grid-row: 1 / span 1; grid-column: 1 / span 1"
                  :title="text('Setup.Misc.Debug.SkipCopy.ToolTip')"
                  id="PageSetupLauncherMisc-CheckDebugSkipCopy"
                  :label="text('Setup.Misc.Debug.SkipCopy')"
                  v-model="values.CheckDebugSkipCopy"
                />
                <PCheckBox
                  disabled
                  data-control="CheckDebugMode"
                  style="height: 22px; grid-row: 1 / span 1; grid-column: 2 / span 1"
                  :title="text('Setup.Misc.Debug.Mode.ToolTip')"
                  id="PageSetupLauncherMisc-CheckDebugMode"
                  :label="text('Setup.Misc.Debug.Mode')"
                  v-model="values.CheckDebugMode"
                />
                <PCheckBox
                  disabled
                  data-control="CheckDebugDelay"
                  style="height: 22px; grid-row: 1 / span 1; grid-column: 3 / span 1"
                  :title="text('Setup.Misc.Debug.Delay.ToolTip')"
                  id="PageSetupLauncherMisc-CheckDebugDelay"
                  :label="text('Setup.Misc.Debug.Delay')"
                  v-model="values.CheckDebugDelay"
                />
              </div>
            </div>
          </PCard>
        </div>
      </div>
    </div>
  </div>
</template>
