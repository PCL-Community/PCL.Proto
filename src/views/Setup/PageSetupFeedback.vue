<script setup lang="ts">
import { pclImagePath } from '@/api/pclAssets'
import { pclFormatLocale, pclText } from '@/api/pclLocale'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PListItem from '@/components/widget/PListItem.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PHint from '@/components/widget/PHint.vue'
import { openExternal } from '@/util/externalLink'

const repository = 'https://github.com/PCL-Community/PCL.Proto'
const statuses = [
  { key: 'Processing', label: /处理中|in progress/i, icon: 'CommandBlock' },
  { key: 'WaitingProcess', label: /等待处理|待处理|triage/i, icon: 'RedstoneBlock' },
  { key: 'Waiting', label: /等待|waiting/i, icon: 'Anvil' },
  { key: 'Paused', label: /暂停|paused|on hold/i, icon: 'RedstoneLampOff' },
  { key: 'UpNext', label: /即将|up next/i, icon: 'RedstoneLampOn' },
  { key: 'Completed', label: /完成|completed/i, icon: 'Grass' },
  { key: 'Declined', label: /拒绝|declined|wontfix/i, icon: 'CobbleStone' },
  { key: 'Ignored', label: /忽略|ignored/i, icon: 'CobbleStone' },
  { key: 'Duplicate', label: /重复|duplicate/i, icon: 'CobbleStone' },
]
type Feedback = {
  id: number
  title: string
  author: string
  date: string
  url: string
  status: string
}
const issues = ref<Feedback[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
let controller: AbortController | undefined
const groups = computed(() =>
  statuses
    .map((status) => ({
      ...status,
      entries: issues.value.filter((issue) => issue.status === status.key),
    }))
    .filter((group) => group.entries.length),
)
async function load() {
  controller?.abort()
  const request = new AbortController()
  controller = request
  loading.value = true
  error.value = null
  try {
    const pages = await Promise.all(
      [1, 2].map(async (page) => {
        const response = await fetch(
          `https://api.github.com/repos/PCL-Community/PCL.Proto/issues?state=all&sort=created&per_page=100&page=${page}`,
          { signal: request.signal },
        )
        if (!response.ok) throw new Error(`HTTP ${response.status}`)
        const data: unknown = await response.json()
        if (!Array.isArray(data)) throw new Error(pclText('Proto.Feedback.InvalidResponse'))
        return data
      }),
    )
    const result: Feedback[] = []
    for (const issue of pages.flat()) {
      if (
        !issue ||
        issue.pull_request ||
        typeof issue.title !== 'string' ||
        typeof issue.number !== 'number' ||
        typeof issue.html_url !== 'string' ||
        !issue.html_url.startsWith(`${repository}/issues/`)
      )
        continue
      const labels = Array.isArray(issue.labels)
        ? issue.labels
            .map((label: string | { name?: string } | null) =>
              typeof label === 'string' ? label : label?.name || '',
            )
            .join(' ')
        : ''
      result.push({
        id: issue.number,
        title: issue.title,
        author: String(issue.user?.login || ''),
        date: issue.created_at,
        url: issue.html_url,
        status:
          statuses.find((status) => status.label.test(labels))?.key ||
          (issue.state === 'closed' ? 'Completed' : 'WaitingProcess'),
      })
    }
    if (!request.signal.aborted) issues.value = result
  } catch (err) {
    if (!request.signal.aborted)
      error.value = `${text('Setup.Feedback.LoadFailed')}：${String(err)}`
  } finally {
    if (!request.signal.aborted) loading.value = false
  }
}
onMounted(load)
onUnmounted(() => controller?.abort())

const text = pclText
const image = (value: string) => pclImagePath(value, import.meta.env.BASE_URL)
</script>
<template>
  <section class="pcl-page feedback-page">
    <PCard :title="text('Setup.Feedback.Submit.Title')"
      ><div class="submit-content">
        <p>{{ text('Setup.Feedback.Submit.Description') }}</p>
        <PButton type="tint" @click="() => openExternal(`${repository}/issues`)">{{
          text('Setup.Feedback.Submit.Open')
        }}</PButton>
      </div></PCard
    >
    <PCard v-if="loading" hide-title
      ><PLoading :card="false" :loading-text="text('Setup.Feedback.Loading')"
    /></PCard>
    <PHint v-else-if="error" severity="error"
      >{{ error
      }}<PButton inline @click="load">{{
        pclText('Launch.Homepage.LoadFailed.Retry')
      }}</PButton></PHint
    >
    <template v-else>
      <PCard
        v-for="group in groups"
        :key="group.key"
        :title="text(`Setup.Feedback.Status.${group.key}`)"
        :default-fold-status="
          ['Completed', 'Declined', 'Ignored', 'Duplicate'].includes(group.key) ? 'fold' : 'unfold'
        "
      >
        <div class="feedback-list">
          <PListItem
            v-for="issue in group.entries"
            :key="issue.id"
            :title="issue.title"
            :subtitle="`${issue.author} | ${new Date(issue.date).toLocaleDateString(pclFormatLocale)}`"
            :icon="image(`Blocks/${group.icon}.png`)"
            clickable
            @click="() => openExternal(issue.url)"
          />
        </div>
      </PCard>
      <p v-if="!issues.length" class="pcl-empty">{{ pclText('Proto.Empty.Feedback') }}</p>
    </template>
  </section>
</template>
<style scoped>
.submit-content {
  padding: 0 20px 22px;
  display: flex;
  flex-direction: column;
  gap: 15px;
  align-items: start;
}
.feedback-list {
  padding: 0 20px 18px;
}
</style>
