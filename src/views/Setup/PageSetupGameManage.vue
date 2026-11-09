<script setup lang="ts">
import SetupItem from '@/components/widget/SetupItem.vue'
import { pclText } from '@/api/pclLocale'
import { usePageSettings } from '@/composables/usePageSettings'
import PCard from '@/components/widget/PCard.vue'
import PCheckBox from '@/components/widget/PCheckBox.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PErrorRetry from '@/components/widget/PErrorRetry.vue'
import {
  downloadSettingKeys,
  downloadSettings,
  downloadSettingValues,
  downloadSettingsStorage,
} from '@/api/downloadSettings'
import { isTauri } from '@tauri-apps/api/core'
import { onScopeDispose, ref, watch } from 'vue'
import sideTip from '@/composables/sideTip'
const desktop = isTauri()
const { values } = usePageSettings(
  'PageSetupGameManage',
  {
    ComboDownloadHttpMode: '0',
  },
  desktop ? downloadSettingKeys : [],
)
const ready = ref(false)
const loadError = ref<string | null>(null)
let hydrating = false
let timer: ReturnType<typeof setTimeout> | undefined
function applyDownloadSettings() {
  timer = undefined
  void downloadSettingsStorage.write(downloadSettings(values)).catch((error) => {
    console.error('Cannot apply download settings', error)
    sideTip.show(String(error), 'warn')
  })
}
function loadDownloadSettings() {
  void downloadSettingsStorage
    .read()
    .then(({ settings }) => {
      hydrating = true
      Object.assign(values, downloadSettingValues(settings))
      hydrating = false
      ready.value = true
      loadError.value = null
    })
    .catch((error) => {
      loadError.value = String(error)
    })
}
if (desktop) loadDownloadSettings()
watch(
  () => downloadSettings(values),
  () => {
    if (!ready.value || hydrating) return
    clearTimeout(timer)
    timer = setTimeout(applyDownloadSettings, 150)
  },
  { flush: 'sync' },
)
onScopeDispose(() => {
  if (timer) {
    clearTimeout(timer)
    applyDownloadSettings()
  }
})
</script>
<template>
  <div class="pcl-page">
    <PErrorRetry
      :error="loadError || downloadSettingsStorage.errors.get(null)"
      @retry="ready ? applyDownloadSettings() : loadDownloadSettings()"
    />

    <PCard :title="pclText('Setup.GameManage.Source.Title')">
      <div class="pcl-card-body">
        <div class="settings-grid">
          <SetupItem :label="pclText('Setup.GameManage.Source.File')" v-slot="{ id }">
            <PComboBox
              :input-id="id"
              :disabled="!ready"
              :options="[
                { key: '0', text: pclText('Setup.GameManage.Source.UseMirror') },
                { key: '1', text: pclText('Setup.GameManage.Source.PreferOfficialThenMirror') },
                { key: '2', text: pclText('Setup.GameManage.Source.UseOfficial') },
              ]"
              v-model="values.ComboDownloadSource"
            />
          </SetupItem>
          <SetupItem :label="pclText('Setup.GameManage.Source.Version')" v-slot="{ id }">
            <PComboBox
              :input-id="id"
              :disabled="!ready"
              :options="[
                { key: '0', text: pclText('Setup.GameManage.Source.Version.Mirror') },
                { key: '1', text: pclText('Setup.GameManage.Source.PreferOfficialThenMirror') },
                { key: '2', text: pclText('Setup.GameManage.Source.UseOfficial') },
              ]"
              v-model="values.ComboDownloadVersion"
            />
          </SetupItem>
          <SetupItem :label="pclText('Setup.GameManage.Download.HttpProtocol')" v-slot="{ id }">
            <PComboBox
              :input-id="id"
              :disabled="!ready"
              :options="[
                {
                  key: '0',
                  text: pclText('Setup.GameManage.Download.HttpProtocol.Auto'),
                  title: pclText('Setup.GameManage.Download.HttpProtocol.Auto.ToolTip'),
                },
                {
                  key: '1',
                  text: pclText('Setup.GameManage.Download.HttpProtocol.Http11'),
                  title: pclText('Setup.GameManage.Download.HttpProtocol.Http11.ToolTip'),
                },
                {
                  key: '2',
                  text: pclText('Setup.GameManage.Download.HttpProtocol.Http2'),
                  title: pclText('Setup.GameManage.Download.HttpProtocol.Http2.ToolTip'),
                },
              ]"
              v-model="values.ComboDownloadHttpMode"
            />
          </SetupItem>
          <SetupItem :label="pclText('Setup.GameManage.Download.Threads')" v-slot="{ id }">
            <input
              :id="id"
              :disabled="!ready"
              :title="pclText('Setup.GameManage.Download.Threads.ToolTip')"
              type="range"
              min="0"
              max="255"
              v-model.number="values['SliderDownloadThread']"
            />
          </SetupItem>
          <SetupItem :label="pclText('Setup.GameManage.Download.FileConnections')" v-slot="{ id }">
            <input
              :id="id"
              :disabled="!ready"
              :title="pclText('Setup.GameManage.Download.FileConnections.ToolTip')"
              type="range"
              min="0"
              max="7"
              v-model.number="values['SliderDownloadFileConnection']"
            />
          </SetupItem>
          <SetupItem :label="pclText('Setup.GameManage.Download.SpeedLimit')" v-slot="{ id }">
            <input
              :id="id"
              :disabled="!ready"
              :title="pclText('Setup.GameManage.Download.SpeedLimit.ToolTip')"
              type="range"
              min="0"
              max="42"
              v-model.number="values['SliderDownloadSpeed']"
            />
          </SetupItem>
          <SetupItem>
            <template #label>{{ pclText('Setup.GameManage.Download.TargetFolder') }}</template>
            <span class="pcl-text" style="opacity: 0.5">{{
              pclText('Setup.GameManage.Download.TargetFolder.Hint')
            }}</span>
          </SetupItem>
          <SetupItem>
            <template #label>{{ pclText('Setup.GameManage.Download.InstallBehavior') }}</template>
            <div class="download-install-options">
              <PCheckBox
                :disabled="!desktop || !ready"
                id="PageSetupGameManage-CheckDownloadAutoSelectVersion"
                :label="pclText('Setup.GameManage.Download.AutoSelectInstance')"
                v-model="values.CheckDownloadAutoSelectVersion"
              />
              <PCheckBox
                disabled
                :title="pclText('Setup.GameManage.Download.FixAuthlib.ToolTip')"
                id="PageSetupGameManage-CheckFixAuthlib"
                :label="pclText('Setup.GameManage.Download.FixAuthlib')"
                v-model="values.CheckFixAuthlib"
              />
            </div>
          </SetupItem>
        </div>
      </div>
    </PCard>
    <PCard :title="pclText('Setup.GameManage.Community.Title')">
      <div class="pcl-card-body">
        <div class="settings-grid">
          <SetupItem :label="pclText('Setup.GameManage.Community.Source')" v-slot="{ id }">
            <PComboBox
              :input-id="id"
              :disabled="!ready"
              :options="[
                { key: '0', text: pclText('Setup.GameManage.Source.UseMirror') },
                {
                  key: '1',
                  text: pclText('Setup.GameManage.Community.Source.PreferOfficialWithFallback'),
                },
                { key: '2', text: pclText('Setup.GameManage.Source.UseOfficial') },
              ]"
              v-model="values.ComboDownloadMod"
            />
          </SetupItem>
          <SetupItem :label="pclText('Setup.GameManage.Community.FilenameFormat')" v-slot="{ id }">
            <PComboBox
              :input-id="id"
              disabled
              :title="pclText('Setup.GameManage.Community.FilenameFormat.ToolTip')"
              :options="[
                { key: '0', text: pclText('Setup.GameManage.Community.FilenameFormat.Style0') },
                { key: '1', text: pclText('Setup.GameManage.Community.FilenameFormat.Style1') },
                { key: '2', text: pclText('Setup.GameManage.Community.FilenameFormat.Style2') },
                { key: '3', text: pclText('Setup.GameManage.Community.FilenameFormat.Style3') },
                { key: '4', text: pclText('Setup.GameManage.Community.FilenameFormat.Style4') },
              ]"
              v-model="values.ComboDownloadTranslateV2"
            />
          </SetupItem>
          <SetupItem :label="pclText('Setup.GameManage.Community.ModManageStyle')" v-slot="{ id }">
            <PComboBox
              :input-id="id"
              disabled
              :title="pclText('Setup.GameManage.Community.ModManageStyle.ToolTip')"
              :options="[
                {
                  key: '0',
                  text: pclText('Setup.GameManage.Community.ModManageStyle.TitleTranslation'),
                },
                {
                  key: '1',
                  text: pclText('Setup.GameManage.Community.ModManageStyle.TitleFilename'),
                },
              ]"
              v-model="values.ComboModLocalNameStyle"
            />
          </SetupItem>
          <SetupItem :label="pclText('Setup.GameManage.Community.QuickDownload')" v-slot="{ id }">
            <PComboBox
              :input-id="id"
              :title="pclText('Setup.GameManage.Community.QuickDownload.ToolTip')"
              :options="[
                { key: '0', text: pclText('Setup.GameManage.Community.QuickDownload.AlwaysAsk') },
                {
                  key: '1',
                  text: pclText('Setup.GameManage.Community.QuickDownload.CurrentInstance'),
                },
                {
                  key: '2',
                  text: pclText('Setup.GameManage.Community.QuickDownload.AskInstance'),
                },
                { key: '3', text: pclText('Setup.GameManage.Community.QuickDownload.AskPath') },
              ]"
              v-model="values.ComboDownloadQuickBehavior"
            />
          </SetupItem>
          <div class="community-checks">
            <PCheckBox
              disabled
              :title="pclText('Setup.GameManage.Community.HideQuilt.ToolTip')"
              id="PageSetupGameManage-CheckDownloadIgnoreQuilt"
              :label="pclText('Setup.GameManage.Community.HideQuilt')"
              v-model="values.CheckDownloadIgnoreQuilt"
            />
            <PCheckBox
              disabled
              :title="pclText('Setup.GameManage.Community.AutoInstallDependencies.ToolTip')"
              id="PageSetupGameManage-CheckDownloadAutoInstallDependencies"
              :label="pclText('Setup.GameManage.Community.AutoInstallDependencies')"
              v-model="values.CheckDownloadAutoInstallDependencies"
            />
          </div>
        </div>
      </div>
    </PCard>
    <PCard :title="pclText('Setup.GameManage.Accessibility.Title')">
      <div class="pcl-card-body settings-grid settings-check-grid">
        <SetupItem>
          <template #label>{{
            pclText('Setup.GameManage.Accessibility.GameUpdateNotice')
          }}</template>
          <div class="download-install-options">
            <PCheckBox
              disabled
              :title="pclText('Setup.GameManage.Accessibility.ReleaseNotice.ToolTip')"
              id="PageSetupGameManage-CheckUpdateRelease"
              :label="pclText('Setup.GameManage.Accessibility.ReleaseNotice')"
              v-model="values.CheckUpdateRelease"
            />
            <PCheckBox
              disabled
              :title="pclText('Setup.GameManage.Accessibility.SnapshotNotice.ToolTip')"
              id="PageSetupGameManage-CheckUpdateSnapshot"
              :label="pclText('Setup.GameManage.Accessibility.SnapshotNotice')"
              v-model="values.CheckUpdateSnapshot"
            />
          </div>
        </SetupItem>
        <SetupItem>
          <template #label>{{ pclText('Setup.GameManage.Accessibility.GameLanguage') }}</template>
          <PCheckBox
            disabled
            id="PageSetupGameManage-CheckHelpLauncherLanguage"
            :label="pclText('Setup.GameManage.Accessibility.AutoLauncherLanguage')"
            v-model="values.CheckHelpLauncherLanguage"
          />
        </SetupItem>
        <PCheckBox
          disabled
          style="grid-column: 1 / -1"
          :title="pclText('Setup.GameManage.Accessibility.ReadClipboard.ToolTip')"
          id="PageSetupGameManage-CheckDownloadClipboard"
          :label="pclText('Setup.GameManage.Accessibility.ReadClipboard')"
          v-model="values.CheckDownloadClipboard"
        />
      </div>
    </PCard>
  </div>
</template>

<style scoped>
.settings-check-grid {
  row-gap: 0;
}
.community-checks {
  grid-column: 1 / -1;
  display: grid;
}
.download-install-options {
  display: flex;
  flex-wrap: wrap;
  gap: 0 20px;
}
</style>
