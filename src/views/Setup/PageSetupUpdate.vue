<script setup lang="ts">
import { pclText } from '@/api/pclLocale'
import { computed, onUnmounted, ref } from 'vue'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PHint from '@/components/widget/PHint.vue'
import appIcon from '/PCL.Proto.svg?url'
import { openExternal } from '@/util/externalLink'
const appVersion = __APP_VERSION__

const repository = 'https://github.com/PCL-Community/PCL.Proto'
const latest = ref<{ version: string; body: string; url: string }>()
const loading = ref(false)
const error = ref<string | null>(null)
const status = computed(() =>
  loading.value
    ? pclText('Proto.Update.Checking')
    : latest.value
      ? latest.value.version.replace(/^v/, '') === appVersion
        ? pclText('Proto.Update.Current')
        : pclText('Proto.Update.Latest', [latest.value.version])
      : pclText('Proto.Update.NotChecked'),
)
let controller: AbortController | undefined
async function check() {
  controller?.abort()
  const request = new AbortController()
  controller = request
  loading.value = true
  error.value = null
  try {
    const response = await fetch(
      'https://api.github.com/repos/PCL-Community/PCL.Proto/releases/latest',
      { signal: request.signal },
    )
    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    const data: unknown = await response.json()
    if (
      !data ||
      typeof data !== 'object' ||
      !('tag_name' in data) ||
      typeof data.tag_name !== 'string' ||
      !('html_url' in data) ||
      typeof data.html_url !== 'string' ||
      !data.html_url.startsWith(`${repository}/releases/`)
    )
      throw new Error(pclText('Proto.Update.InvalidResponse'))
    if (!request.signal.aborted)
      latest.value = {
        version: data.tag_name,
        url: data.html_url,
        body: 'body' in data && typeof data.body === 'string' ? data.body : '',
      }
  } catch (err) {
    if (!request.signal.aborted) error.value = pclText('Proto.Update.Failed', [String(err)])
  } finally {
    if (!request.signal.aborted) loading.value = false
  }
}
onUnmounted(() => controller?.abort())

const text = pclText
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
        <PButton type="tint" :disabled="loading" @click="check">{{
          text('Setup.Update.CheckAgain')
        }}</PButton
        ><PButton @click="() => openExternal(latest?.url || `${repository}/releases`)">{{
          text('Setup.Update.Changelog.View')
        }}</PButton>
      </div></PCard
    >
    <PHint v-if="error" severity="error">{{ error }}</PHint>
    <PCard :title="text('Setup.Update.Changelog.View')"
      ><div class="changelog">
        <p>
          {{
            latest
              ? latest.body || pclText('Proto.Update.NoNotes')
              : pclText('Proto.Update.NotesPlaceholder')
          }}
        </p>
        <PButton inline @click="() => openExternal(latest?.url || `${repository}/releases`)">{{
          text('Setup.Update.Changelog.MoreInfo')
        }}</PButton
        ><PButton v-if="latest" type="tint" @click="() => openExternal(latest?.url)">{{
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
.changelog p {
  white-space: pre-wrap;
  user-select: text;
}
</style>
