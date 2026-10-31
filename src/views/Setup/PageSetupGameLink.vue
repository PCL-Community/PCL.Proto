<script setup lang="ts">
import { pclText } from '@/api/pclLocale'
import { usePageSettings } from '@/composables/usePageSettings'
import PCard from '@/components/widget/PCard.vue'
import PCheckBox from '@/components/widget/PCheckBox.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PHint from '@/components/widget/PHint.vue'
import PTextBox from '@/components/widget/PTextBox.vue'
const { values } = usePageSettings('PageSetupGameLink', {
  ComboPreferProtocol: '0',
})

const text = pclText
</script>
<template>
  <div class="pcl-page">
    <p class="settings-unavailable">{{ text('Proto.Settings.Unavailable') }}</p>
    <div>
      <div data-control="PanBack">
        <div
          data-control="PanMain"
          style="margin: 10px 25px; display: flex; flex-direction: column"
        >
          <PCard style="margin: 15px 0px" :title="text('EasyTier')">
            <div style="margin: 0px 25px 15px 25px; display: flex; flex-direction: column">
              <PHint style="margin: 0px 0px 10px 0px" severity="warning">{{
                text('Setup.GameLink.RestartRequired')
              }}</PHint>
              <div
                style="
                  display: grid;
                  grid-template-columns: auto 8px minmax(0, 1fr);
                  grid-template-rows: 28px 8px 28px 4px 28px 28px 28px 28px;
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
                  >{{ text('Setup.GameLink.Username') }}</span
                >
                <PTextBox
                  disabled
                  data-control="TextLinkUsername"
                  style="grid-row: 1 / span 1; grid-column: 3 / span 2"
                  :title="text('Setup.GameLink.Username.ToolTip')"
                  :placeholder="text('')"
                  v-model="values.TextLinkUsername"
                />
                <span
                  style="
                    grid-row: 3 / span 1;
                    grid-column: 1 / span 1;
                    justify-self: start;
                    align-self: center;
                  "
                  class="pcl-text"
                  >{{ text('Setup.GameLink.Protocol.Preference') }}</span
                >
                <PComboBox
                  disabled
                  data-control="ComboPreferProtocol"
                  style="grid-row: 3 / span 1; grid-column: 3 / span 1"
                  :options="[
                    {
                      key: '0',
                      text: text('Setup.GameLink.Protocol.Tcp'),
                      title: text('Setup.GameLink.Protocol.Tcp.ToolTip'),
                    },
                    {
                      key: '1',
                      text: text('Setup.GameLink.Protocol.Udp'),
                      title: text('Setup.GameLink.Protocol.Udp.ToolTip'),
                    },
                  ]"
                  v-model="values.ComboPreferProtocol"
                />
                <PCheckBox
                  disabled
                  data-control="CheckLatencyFirstMode"
                  style="grid-row: 5 / span 1; grid-column: 1 / span 3"
                  :title="text('Setup.GameLink.LatencyFirstMode.ToolTip')"
                  id="PageSetupGameLink-CheckLatencyFirstMode"
                  :label="text('Setup.GameLink.LatencyFirstMode')"
                  v-model="values.CheckLatencyFirstMode"
                />
                <PCheckBox
                  disabled
                  data-control="CheckTryPunchSym"
                  style="grid-row: 6 / span 1; grid-column: 1 / span 3"
                  :title="text('Setup.GameLink.PortGuessNat.ToolTip')"
                  id="PageSetupGameLink-CheckTryPunchSym"
                  :label="text('Setup.GameLink.PortGuessNat')"
                  v-model="values.CheckTryPunchSym"
                />
                <PCheckBox
                  disabled
                  data-control="CheckEnableIPv6"
                  style="grid-row: 7 / span 1; grid-column: 1 / span 3"
                  :title="text('Setup.GameLink.EnableIPv6.ToolTip')"
                  id="PageSetupGameLink-CheckEnableIPv6"
                  :label="text('Setup.GameLink.EnableIPv6')"
                  v-model="values.CheckEnableIPv6"
                />
                <PCheckBox
                  disabled
                  data-control="CheckEnableCliOutput"
                  style="grid-row: 8 / span 1; grid-column: 1 / span 3"
                  :title="text('Setup.GameLink.CliDebugOutput.ToolTip')"
                  id="PageSetupGameLink-CheckEnableCliOutput"
                  :label="text('Setup.GameLink.CliDebugOutput')"
                  v-model="values.CheckEnableCliOutput"
                />
              </div>
            </div>
          </PCard>
        </div>
      </div>
    </div>
  </div>
</template>
