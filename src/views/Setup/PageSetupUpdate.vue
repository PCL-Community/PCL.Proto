<script setup lang="ts">
import { pclText } from '@/api/pclLocale'
import { computed, onMounted } from 'vue'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PHint from '@/components/widget/PHint.vue'
import PMarkdown from '@/components/widget/PMarkdown.vue'
import appIcon from '/PCL.Proto.png?url'
import { openExternal } from '@/util/externalLink'
import {
  checkLauncherUpdate,
  launcherUpdate,
  launcherUpdateChecking,
  launcherUpdateError,
} from '@/api/launcherUpdate'
const appVersion = __APP_VERSION__

const repository = 'https://github.com/PCL-Community/PCL.Proto'
const releaseUrl = computed(() => launcherUpdate.value?.url || `${repository}/releases`)
const status = computed(() =>
  launcherUpdateChecking.value
    ? pclText('Proto.Update.Checking')
    : launcherUpdate.value
      ? launcherUpdate.value.version.replace(/^v/, '') === appVersion
        ? pclText('Proto.Update.Current')
        : pclText('Proto.Update.Latest', [launcherUpdate.value.version])
      : pclText('Proto.Update.NotChecked'),
)
onMounted(() => void checkLauncherUpdate())
const openRelease = () => openExternal(releaseUrl.value)
</script>
<template>
  <section class="pcl-page update-page">
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
        ><PButton v-if="launcherUpdate" type="tint" @click="openRelease">{{
          pclText('Proto.Update.Download')
        }}</PButton>
      </div></PCard
    >
  </section>
</template>
<style scoped>
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
