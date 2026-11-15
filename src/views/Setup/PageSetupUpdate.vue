<script setup lang="ts">
import { pclText } from '@/api/pclLocale'
import { computed, onMounted } from 'vue'
import PButton from '@/components/widget/PButton.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PCard from '@/components/widget/PCard.vue'
import PHint from '@/components/widget/PHint.vue'
import PMarkdown from '@/components/widget/PMarkdown.vue'
import appIcon from '/PCL.Proto.png?url'
import { openExternal } from '@/util/externalLink'
import { useModal } from '@/composables/useModal'
import {
  checkLauncherUpdate,
  launcherUpdate,
  launcherUpdateChannel,
  launcherUpdateChecking,
  launcherUpdateError,
  launcherUpdateMode,
  launcherUpdateAvailable,
  setLauncherUpdateChannel,
  setLauncherUpdateMode,
  type UpdateChannel,
  type UpdateMode,
} from '@/api/launcherUpdate'
const appVersion = __APP_VERSION__

const channelOptions = [
  {
    key: 'release',
    text: pclText('Setup.Update.Channel.Release'),
    title: pclText('Setup.Update.Channel.Release.ToolTip'),
  },
  {
    key: 'beta',
    text: pclText('Setup.Update.Channel.Beta'),
    title: pclText('Setup.Update.Channel.Beta.ToolTip'),
  },
]
const modeOptions = [
  {
    key: '2',
    text: pclText('Setup.Update.Auto.NotifyOnly'),
    title: pclText('Setup.Update.Auto.NotifyOnly.ToolTip'),
  },
  {
    key: '3',
    text: pclText('Setup.Update.Auto.Disabled'),
    title: pclText('Setup.Update.Auto.Disabled.ToolTip'),
  },
]
async function selectChannel(value: string | number | undefined) {
  const channel = String(value) as UpdateChannel
  if (channel === launcherUpdateChannel.value) return
  const modal = useModal()
  const result = await modal.open({
    title: pclText('Setup.Update.Channel.Common.Warning.Title'),
    content: pclText('Setup.Update.Channel.Beta.Warning.Message'),
    buttons: [
      {
        type: 'tint',
        content: pclText('Setup.Update.Channel.Common.Warning.Confirm'),
        operation: () => modal.close(true),
      },
      { content: pclText('Common.Action.Cancel'), operation: () => modal.close(false) },
    ],
  })
  if (!result.confirmed) return
  setLauncherUpdateChannel(channel)
  void checkLauncherUpdate()
}
function selectMode(value: string | number | undefined) {
  setLauncherUpdateMode(Number(value) as UpdateMode)
}

const repository = 'https://github.com/PCL-Community/PCL.Proto'
const releaseUrl = computed(() => launcherUpdate.value?.url || `${repository}/releases`)
const status = computed(() =>
  launcherUpdateChecking.value
    ? pclText('Proto.Update.Checking')
    : launcherUpdate.value
      ? launcherUpdateAvailable.value
        ? pclText('Proto.Update.Latest', [launcherUpdate.value.version])
        : pclText('Proto.Update.Current')
      : pclText('Proto.Update.NotChecked'),
)
onMounted(() => void checkLauncherUpdate())
const openRelease = () => openExternal(releaseUrl.value)
</script>
<template>
  <section class="pcl-page update-page">
    <PCard hide-title
      ><div class="update-options">
        <span class="pcl-text">{{ pclText('Setup.Update.Channel.Title') }}</span>
        <PComboBox
          id="PageSetupUpdate-ComboSystemUpdateChannel"
          :options="channelOptions"
          :model-value="launcherUpdateChannel"
          @update:model-value="selectChannel"
        />
        <span class="pcl-text">{{ pclText('Setup.Update.Auto.Title') }}</span>
        <PComboBox
          id="PageSetupUpdate-ComboSystemUpdateMode"
          :options="modeOptions"
          :model-value="String(launcherUpdateMode)"
          @update:model-value="selectMode"
        /></div
    ></PCard>
    <PCard hide-title
      ><div class="version-check">
        <img :src="appIcon" alt="PCL.Proto" />
        <div>
          <h2>PCL.Proto {{ appVersion }}</h2>
          <p>{{ status }}</p>
        </div>
        <PButton type="tint" :disabled="launcherUpdateChecking" @click="checkLauncherUpdate">{{
          pclText('Setup.Update.CheckAgain')
        }}</PButton
        ><PButton @click="openRelease">{{ pclText('Setup.Update.Changelog.View') }}</PButton>
      </div></PCard
    >
    <PHint v-if="launcherUpdateError" severity="error">{{
      pclText('Proto.Update.Failed', [launcherUpdateError])
    }}</PHint>
    <PCard :title="pclText('Setup.Update.Changelog.View')"
      ><div class="changelog">
        <PMarkdown
          :content="
            launcherUpdate
              ? launcherUpdate.body || pclText('Proto.Update.NoNotes')
              : pclText('Proto.Update.NotesPlaceholder')
          "
        />
        <PButton inline @click="openRelease">{{
          pclText('Setup.Update.Changelog.MoreInfo')
        }}</PButton
        ><PButton v-if="launcherUpdateAvailable" type="tint" @click="openRelease">{{
          pclText('Proto.Update.Download')
        }}</PButton>
      </div></PCard
    >
  </section>
</template>
<style scoped>
.update-options {
  padding: 10px 25px 20px;
  display: grid;
  grid-template-columns: minmax(120px, 1fr) minmax(120px, 1fr);
  align-items: center;
  gap: 8px 10px;
}
.version-check {
  padding: 20px;
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.version-check img {
  width: 50px;
  height: 50px;
}
.version-check > div {
  flex: 1;
  min-width: 150px;
}
h2 {
  font-size: 20px;
}
.changelog {
  padding: 0 25px 20px;
  display: flex;
  flex-direction: column;
  align-items: start;
  gap: 12px;
}
</style>
