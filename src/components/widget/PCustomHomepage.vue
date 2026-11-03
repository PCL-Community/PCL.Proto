<script setup lang="ts">
import { computed } from 'vue'
import { pclText, pclFormatLocale, pclLocale } from '@/api/pclLocale'
import { useAccountInfo } from '@/stores/account'
import { useSelectedInstance } from '@/stores/gameLaunch'
import { getProfileTypeLabel } from '@/util/profile'
import { homepageRevision } from '@/api/customHomepage'
import { version } from '../../../package.json'
import { parseHomepage } from '@/util/homepageXaml'
import { renderHomepageNodes } from '@/api/xamlParser'
import { useHomepageActions } from '@/composables/useHomepageActions'
import PHint from './PHint.vue'
import hintsZh from '@/assets/homepage/hints-zh-CN.txt?raw'
import hintsEn from '@/assets/homepage/hints-en-US.txt?raw'
const props = withDefaults(defineProps<{ source: string; preview?: boolean }>(), { preview: false })
const result = computed(() => {
  try {
    return { ...parseHomepage(props.source), error: undefined }
  } catch (error) {
    return { nodes: [], warnings: [], error: String(error) }
  }
})
const action = useHomepageActions(props.preview)
const account = useAccountInfo()
const selected = useSelectedInstance()
const hints = computed(() =>
  (pclLocale.value.startsWith('en') ? hintsEn : hintsZh)
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line && !/pcl|hints\.txt/i.test(line)),
)
const hint = computed(() => {
  void homepageRevision.value
  const entries = hints.value
  return entries[Math.floor(Math.random() * entries.length)] || ''
})
const variables = computed(() => ({
  hint: hint.value,
  user: account.profile?.userName || '',
  uuid: account.profile?.uuid || '',
  login: account.profile ? getProfileTypeLabel(account.profile) : '',
  name: selected.instance_info?.name || '',
  version: selected.instance_info?.version || '',
  version_path: selected.instance_info?.directory || '',
  verpath: selected.instance_info?.directory || '',
  minecraft: selected.instance_info?.global_dir || '',
  pcl_version: version,
  date: new Date().toLocaleDateString(pclFormatLocale.value),
  time: new Date().toLocaleTimeString(pclFormatLocale.value),
}))
const Content = () => renderHomepageNodes(result.value.nodes, action, variables.value)
</script>
<template>
  <div class="custom-homepage">
    <PHint v-if="result.error" severity="error">{{ result.error }}</PHint>
    <PHint v-if="result.warnings.length" severity="warning">
      {{ pclText('Proto.Homepage.Unsupported', [result.warnings.join('、')]) }}
    </PHint>
    <Content v-if="!result.error" />
  </div>
</template>
<style>
.custom-homepage {
  display: flex;
  flex-direction: column;
  gap: 15px;
  min-width: 0;
}
.homepage-card-content {
  padding: 0 25px 20px;
}
.custom-homepage p {
  user-select: text;
}
.custom-homepage .mybutton svg {
  vertical-align: middle;
  margin-right: 6px;
}
</style>
