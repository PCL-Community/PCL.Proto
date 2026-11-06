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
</script>
<template>
  <div class="pcl-page">
    <PCard :title="pclText('EasyTier')">
      <div style="margin: 0px 25px 15px 25px; display: flex; flex-direction: column">
        <PHint style="margin: 0px 0px 10px 0px" severity="warning">{{
          pclText('Setup.GameLink.RestartRequired')
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
            >{{ pclText('Setup.GameLink.Username') }}</span
          >
          <PTextBox
            disabled
            style="grid-row: 1 / span 1; grid-column: 3 / span 2"
            :title="pclText('Setup.GameLink.Username.ToolTip')"
            :placeholder="pclText('')"
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
            >{{ pclText('Setup.GameLink.Protocol.Preference') }}</span
          >
          <PComboBox
            disabled
            style="grid-row: 3 / span 1; grid-column: 3 / span 1"
            :options="[
              {
                key: '0',
                text: pclText('Setup.GameLink.Protocol.Tcp'),
                title: pclText('Setup.GameLink.Protocol.Tcp.ToolTip'),
              },
              {
                key: '1',
                text: pclText('Setup.GameLink.Protocol.Udp'),
                title: pclText('Setup.GameLink.Protocol.Udp.ToolTip'),
              },
            ]"
            v-model="values.ComboPreferProtocol"
          />
          <PCheckBox
            disabled
            style="grid-row: 5 / span 1; grid-column: 1 / span 3"
            :title="pclText('Setup.GameLink.LatencyFirstMode.ToolTip')"
            id="PageSetupGameLink-CheckLatencyFirstMode"
            :label="pclText('Setup.GameLink.LatencyFirstMode')"
            v-model="values.CheckLatencyFirstMode"
          />
          <PCheckBox
            disabled
            style="grid-row: 6 / span 1; grid-column: 1 / span 3"
            :title="pclText('Setup.GameLink.PortGuessNat.ToolTip')"
            id="PageSetupGameLink-CheckTryPunchSym"
            :label="pclText('Setup.GameLink.PortGuessNat')"
            v-model="values.CheckTryPunchSym"
          />
          <PCheckBox
            disabled
            style="grid-row: 7 / span 1; grid-column: 1 / span 3"
            :title="pclText('Setup.GameLink.EnableIPv6.ToolTip')"
            id="PageSetupGameLink-CheckEnableIPv6"
            :label="pclText('Setup.GameLink.EnableIPv6')"
            v-model="values.CheckEnableIPv6"
          />
          <PCheckBox
            disabled
            style="grid-row: 8 / span 1; grid-column: 1 / span 3"
            :title="pclText('Setup.GameLink.CliDebugOutput.ToolTip')"
            id="PageSetupGameLink-CheckEnableCliOutput"
            :label="pclText('Setup.GameLink.CliDebugOutput')"
            v-model="values.CheckEnableCliOutput"
          />
        </div>
      </div>
    </PCard>
  </div>
</template>
