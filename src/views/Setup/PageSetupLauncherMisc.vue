<script setup lang="ts">
import PErrorRetry from '@/components/widget/PErrorRetry.vue'
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
  if (!proxyError.value) sideTip.show(pclText('Proto.Settings.ProxySaved'), 'success')
}
</script>
<template>
  <div class="pcl-page">
    <PErrorRetry :error="logSaveError" @retry="saveLogSettings" />
    <PErrorRetry :error="proxyError" @retry="saveProxy()" />

    <PCard :title="pclText('Setup.Misc.System.Title')">
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
          >{{ pclText('Setup.Misc.System.Announcement') }}</span
        >
        <PComboBox
          disabled
          style="grid-row: 1 / span 1; grid-column: 2 / span 1"
          :options="[
            {
              key: '0',
              text: pclText('Setup.Misc.System.Announcement.All'),
              title: pclText('Setup.Misc.System.Announcement.All.ToolTip'),
            },
            { key: '1', text: pclText('Setup.Misc.System.Announcement.ImportantOnly') },
            { key: '2', text: pclText('Setup.Misc.System.Announcement.Disabled') },
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
          >{{ pclText('Setup.Misc.System.MaxFps') }}</span
        >
        <input
          disabled
          style="grid-row: 3 / span 1; grid-column: 2 / span 1"
          :title="pclText('Setup.Misc.System.MaxFps.ToolTip')"
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
          >{{ pclText('Setup.Misc.System.MaxLogLines') }}</span
        >
        <input
          style="grid-row: 5 / span 1; grid-column: 2 / span 1"
          :title="pclText('Setup.Misc.System.MaxLogLines.ToolTip')"
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
            style="grid-row: 1 / span 1; grid-column: 1 / span 1"
            id="PageSetupLauncherMisc-CheckSystemDisableHardwareAcceleration"
            :label="pclText('Setup.Misc.System.DisableHardwareAcceleration')"
            v-model="values.CheckSystemDisableHardwareAcceleration"
          />
          <PCheckBox
            disabled
            style="grid-row: 1 / span 1; grid-column: 2 / span 1"
            :title="pclText('Setup.Misc.System.Telemetry.ToolTip')"
            id="PageSetupLauncherMisc-CheckSystemTelemetry"
            :label="pclText('Setup.Misc.System.Telemetry')"
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
            style="
              min-width: 140px;
              margin: 0px 20px 0px 0px;
              padding: 0px 13px;
              grid-row: 1 / span 1;
              grid-column: 1 / span 1;
            "
            @click="showUnavailable"
            >{{ pclText('Setup.Misc.System.ExportSettings') }}</PButton
          >
          <PButton
            style="
              min-width: 140px;
              margin: 0px 20px 0px 0px;
              padding: 0px 13px;
              grid-row: 1 / span 1;
              grid-column: 2 / span 1;
            "
            @click="showUnavailable"
            >{{ pclText('Setup.Misc.System.ImportSettings') }}</PButton
          >
          <PButton
            style="
              min-width: 140px;
              margin: 0px 20px 0px 0px;
              padding: 0px 13px;
              grid-row: 1 / span 1;
              grid-column: 3 / span 1;
            "
            @click="() => showUnavailable()"
            type="warn"
            >{{ pclText('Setup.Misc.System.StopUsingPclCe') }}</PButton
          >
        </div>
      </div>
    </PCard>
    <PCard :title="pclText('Setup.Misc.Network.Title')">
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
          style="grid-row: 1 / span 1; grid-column: 1 / span 2"
          :title="pclText('Setup.Misc.Network.DoH.ToolTip')"
          id="PageSetupLauncherMisc-CheckNetDohEnable"
          :label="pclText('Setup.Misc.Network.DoH')"
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
          >{{ pclText('Setup.Misc.Network.HttpProxy') }}</span
        >
        <div
          style="
            grid-row: 3 / span 1;
            grid-column: 2 / span 1;
            display: grid;
            grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) minmax(0, 1fr) minmax(0, 0.2fr);
          "
        >
          <label
            style="height: 22px; grid-row: 1 / span 1; grid-column: 1 / span 1"
            class="pcl-radio"
            ><input :disabled="!proxyReady" type="radio" v-model="proxyMode" value="off" />{{
              pclText('Setup.Misc.Network.Proxy.None')
            }}</label
          >
          <label
            style="height: 22px; grid-row: 1 / span 1; grid-column: 2 / span 1"
            class="pcl-radio"
            ><input :disabled="!proxyReady" type="radio" v-model="proxyMode" value="system" />{{
              pclText('Setup.Misc.Network.Proxy.System')
            }}</label
          >
          <label
            style="height: 22px; grid-row: 1 / span 1; grid-column: 3 / span 1"
            class="pcl-radio"
            ><input :disabled="!proxyReady" type="radio" v-model="proxyMode" value="custom" />{{
              pclText('Setup.Misc.Network.Proxy.Custom')
            }}</label
          >
        </div>
        <div
          style="
            grid-row: 5 / span 1;
            grid-column: 2 / span 1;
            display: grid;
            grid-template-columns: auto minmax(0, 1fr) auto minmax(0, 1fr) auto;
            grid-template-rows: 28px 9px 28px 9px 28px;
          "
        >
          <PHint style="grid-row: 1 / span 1; grid-column: 1 / span 5" severity="warning">{{
            pclText('Setup.Misc.Network.Proxy.UntrustedWarning')
          }}</PHint>
          <PTextBox
            style="grid-row: 3 / span 1; grid-column: 1 / span 5"
            :disabled="!proxyReady || proxyMode !== 'custom'"
            :title="pclText('Setup.Misc.Network.Proxy.InvalidWarning')"
            :placeholder="pclText('Proto.Settings.ProxyExample')"
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
            >{{ pclText('Setup.Misc.Network.Proxy.Username') }}</span
          >
          <PTextBox
            disabled
            style="margin: 0px 5px 0px 0px; grid-row: 5 / span 1; grid-column: 2 / span 1"
            :title="pclText('Proto.Settings.ProxyCredentialsHint')"
            :placeholder="pclText('Setup.Misc.Network.Proxy.Optional')"
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
            >{{ pclText('Setup.Misc.Network.Proxy.Password') }}</span
          >
          <PTextBox
            disabled
            type="password"
            style="margin: 0px 5px 0px 0px; grid-row: 5 / span 1; grid-column: 4 / span 1"
            :title="pclText('Proto.Settings.ProxyCredentialsHint')"
            :placeholder="pclText('Setup.Misc.Network.Proxy.Optional')"
            v-model="values.TextSystemHttpProxyCustomPassword"
          />
          <PButton
            style="grid-row: 5 / span 1; grid-column: 5 / span 1"
            :disabled="!proxyReady || proxyBusy"
            @click="applyProxy"
            >{{ pclText('Setup.Misc.Network.Proxy.Apply') }}</PButton
          >
        </div>
      </div>
    </PCard>
    <PCard :title="pclText('Setup.Misc.Debug.Title')" default-fold-status="fold">
      <div style="margin: 0px 25px 15px 25px; display: flex; flex-direction: column">
        <div
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
            >{{ pclText('Setup.Misc.Debug.AnimationSpeed') }}</span
          >
          <input
            disabled
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
            style="height: 22px; grid-row: 1 / span 1; grid-column: 1 / span 1"
            :title="pclText('Setup.Misc.Debug.SkipCopy.ToolTip')"
            id="PageSetupLauncherMisc-CheckDebugSkipCopy"
            :label="pclText('Setup.Misc.Debug.SkipCopy')"
            v-model="values.CheckDebugSkipCopy"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 1 / span 1; grid-column: 2 / span 1"
            :title="pclText('Setup.Misc.Debug.Mode.ToolTip')"
            id="PageSetupLauncherMisc-CheckDebugMode"
            :label="pclText('Setup.Misc.Debug.Mode')"
            v-model="values.CheckDebugMode"
          />
          <PCheckBox
            disabled
            style="height: 22px; grid-row: 1 / span 1; grid-column: 3 / span 1"
            :title="pclText('Setup.Misc.Debug.Delay.ToolTip')"
            id="PageSetupLauncherMisc-CheckDebugDelay"
            :label="pclText('Setup.Misc.Debug.Delay')"
            v-model="values.CheckDebugDelay"
          />
        </div>
      </div>
    </PCard>
  </div>
</template>
