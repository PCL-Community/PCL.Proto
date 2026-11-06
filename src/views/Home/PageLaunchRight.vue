<script setup lang="ts">
import { pclText } from '@/api/pclLocale'
import { computed, defineAsyncComponent, onBeforeUnmount, ref, watch } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { appearance } from '@/api/appearance'
import {
  cachedHomepage,
  downloadHomepage,
  homepagePresets,
  homepageRevision,
  readLocalHomepage,
} from '@/api/customHomepage'
import { parseHomepage } from '@/util/homepageXaml'
import PCustomHomepage from '@/components/widget/PCustomHomepage.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PageHomepageNewsView from './PageHomepageNewsView.vue'
import PHint from '@/components/widget/PHint.vue'
import PCard from '@/components/widget/PCard.vue'
defineProps<{ launchLog?: string }>()

const text = pclText
const website = !isTauri()
const ComponentShowcase = defineAsyncComponent(
  () => import('@/components/website/ComponentShowcase.vue'),
)
const mode = computed(() => appearance.values.UiCustomType)
const welcome = computed(
  () => mode.value === '3' && appearance.values.ComboCustomPreset === 'proto',
)
const officialFeed = computed(
  () => mode.value === '3' && appearance.values.ComboCustomPreset === '15',
)
const source = ref<string>()
const error = ref<string>()
const loading = ref(false)
let controller: AbortController | undefined
let generation = 0
watch(
  [
    mode,
    () => appearance.values.TextCustomNet,
    () => appearance.values.ComboCustomPreset,
    homepageRevision,
  ],
  async (_, previous) => {
    controller?.abort()
    const request = ++generation
    controller = new AbortController()
    const signal = controller.signal
    error.value = undefined
    const sameSource =
      previous &&
      mode.value === previous[0] &&
      appearance.values.TextCustomNet === previous[1] &&
      appearance.values.ComboCustomPreset === previous[2]
    if (!sameSource) source.value = undefined
    if (website || mode.value === '0' || welcome.value || officialFeed.value) {
      loading.value = false
      return
    }
    loading.value = true
    try {
      let content: string
      if (mode.value === '3' && appearance.values.ComboCustomPreset === '0') {
        content =
          '<local:MyCard Title="{DynamicResource Launch.Status.Trivia}"><StackPanel><TextBlock Text="{hint}" Margin="0,0,0,12"/><local:MyTextButton Text="{DynamicResource Setup.Ui.Homepage.Refresh}" EventType="RefreshHomepage"/></StackPanel></local:MyCard>'
      } else if (mode.value === '1') content = await readLocalHomepage()
      else {
        const url =
          mode.value === '2'
            ? appearance.values.TextCustomNet
            : homepagePresets.find((preset) => preset.key === appearance.values.ComboCustomPreset)
                ?.url
        if (!url) throw new Error(text('Proto.Homepage.ChoosePreset'))
        if (!source.value) source.value = cachedHomepage(url)
        content = await downloadHomepage(url, signal)
      }
      parseHomepage(content)
      if (request === generation) source.value = content
    } catch (failure) {
      if (request === generation && !signal.aborted) error.value = String(failure)
    } finally {
      if (request === generation) loading.value = false
    }
  },
  { immediate: true },
)
onBeforeUnmount(() => {
  generation++
  controller?.abort()
})
</script>
<template>
  <section class="pcl-page">
    <PCard
      v-if="website || welcome"
      :title="text('Proto.About.Welcome')"
      default-fold-status="unfold"
    >
      <div class="proto-introduction">
        <PHint severity="info">{{ text('Proto.About.Intro') }}</PHint>
        <p>{{ text('Proto.About.Description') }}</p>
      </div>
    </PCard>
    <template v-if="!website && mode !== '0' && !welcome">
      <PLoading
        v-if="loading && source === undefined"
        :loading-text="text('Launch.Homepage.Loading')"
      />
      <PHint v-if="error" severity="error">{{ error }}</PHint>
      <PageHomepageNewsView v-if="officialFeed" embedded />
      <PCustomHomepage v-if="source !== undefined" :source="source" />
    </template>
    <PCard v-if="launchLog" :title="text('Launch.Right.Log.Title')">
      <pre class="launch-log">{{ launchLog }}</pre>
    </PCard>
    <ComponentShowcase v-if="website" />
  </section>
</template>
<style scoped>
.proto-introduction {
  padding: 0 25px 15px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.launch-log {
  padding: 0 20px 18px;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  user-select: text;
}
</style>
