<script setup lang="ts">
import { pclText } from '@/api/pclLocale'
import { computed, onBeforeUnmount, ref, watch } from 'vue'
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
import PButton from '@/components/widget/PButton.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PageHomepageNewsView from './PageHomepageNewsView.vue'
import PHint from '@/components/widget/PHint.vue'
import PCard from '@/components/widget/PCard.vue'
defineProps<{ launchLog?: string }>()

const text = pclText
const mode = computed(() => appearance.values.UiCustomType)
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
    if (mode.value === '0' || officialFeed.value) {
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
    <template v-if="mode !== '0'">
      <PLoading
        v-if="loading && source === undefined"
        :loading-text="text('Launch.Homepage.Loading')"
      />
      <PHint v-if="error" severity="error">{{ error }}</PHint>
      <div class="homepage-actions">
        <PButton compact :disabled="loading" @click="homepageRevision++">{{
          text('Setup.Ui.Homepage.Refresh')
        }}</PButton>
        <PButton compact @click="$router.push('/homepageeditor')">{{
          text('Proto.Title.HomepageEditor')
        }}</PButton>
      </div>
      <PageHomepageNewsView v-if="officialFeed" embedded />
      <PCustomHomepage v-if="source !== undefined" :source="source" />
    </template>
    <PCard v-if="launchLog" :title="text('Launch.Right.Log.Title')">
      <pre class="launch-log">{{ launchLog }}</pre>
    </PCard>
  </section>
</template>
<style scoped>
.homepage-actions {
  display: flex;
  gap: 10px;
}
.launch-log {
  padding: 0 20px 18px;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  user-select: text;
}
</style>
