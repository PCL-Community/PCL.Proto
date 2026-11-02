<script setup lang="ts">
import { compProjectUrl, compSourceOf, getCompProject, getCompVersions } from '@/api/compProjects'
import type { IProject, IVersion, CompSource } from '@/types/comp'
import PCard from '@/components/widget/PCard.vue'
import PVirtualList from '@/components/widget/PVirtualList.vue'
import {
  compVersionFilters,
  groupCompVersions,
  groupedVersionName,
  loaderNames,
  selectedCompVersions,
} from '@/api/compResources'
import PCompItem from '@/components/widget/PCompItem.vue'
import PListItem from '@/components/widget/PListItem.vue'
import PButton from '@/components/widget/PButton.vue'
import PIconTextButton from '@/components/widget/PIconTextButton.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PHint from '@/components/widget/PHint.vue'
import { useRoute } from 'vue-router'
import { computed, onScopeDispose, ref, shallowRef, watch } from 'vue'
import { pclFormatLocale, pclText } from '@/api/pclLocale'
import sideTip from '@/composables/sideTip'
import { useCompFavorites } from '@/stores/compFavorites'
import { openExternal } from '@/util/externalLink'
import { datapackTargetFromQuery, saveResource } from '@/api/compDownload'
const favorites = useCompFavorites()

const route = useRoute()
const text = pclText
const project = shallowRef<IProject>()
const versions = shallowRef<IVersion[]>([])
/// 当前详情的来源：CF 的工程 ID 是纯数字。
const source = ref<CompSource>('modrinth')
const loading = ref(false)
const error = ref('')
const versionFilter = ref('')
const requestedVersion = ref('')
const requestedLoader = ref('')
const loaderFilter = ref('')
const item = computed(
  () =>
    project.value && {
      ...project.value,
      date_modified: project.value.updated,
      project_id: project.value.id,
      versions: [...new Set(versions.value.flatMap((version) => version.game_versions))],
    },
)
// 筛选选项按 CE 规则逐级合并（1.21.1 → 1.21 → 远古版）
const versionFilters = computed(() =>
  compVersionFilters(versions.value.flatMap((version) => version.game_versions)),
)
const loaders = computed(() => [...new Set(versions.value.flatMap((version) => version.loaders))])
const isMod = computed(() => project.value?.project_type === 'mod')
const groups = computed(() => {
  const normal = groupCompVersions(
    versions.value,
    versionFilter.value,
    loaderFilter.value,
    isMod.value && loaders.value.length > 1,
    versionFilters.value.byDrop,
    versionFilters.value.foldOld,
  )
  const selected = selectedCompVersions(
    versions.value,
    requestedVersion.value,
    requestedLoader.value,
    versionFilter.value,
    loaderFilter.value,
    versionFilters.value.byDrop,
    versionFilters.value.foldOld,
  )
  if (!selected.length) return normal
  const loader = loaderNames[requestedLoader.value] || requestedLoader.value
  return [
    {
      key: 'selected',
      name: text('Download.Comp.Detail.SelectedVersion', [
        loader ? `${loader} ` : '',
        requestedVersion.value,
      ]),
      gameVersion: requestedVersion.value,
      versions: selected,
    },
    ...normal,
  ]
})
const sourceName = computed(() => (source.value === 'curseforge' ? 'CurseForge' : 'Modrinth'))
const projectUrl = computed(() => (project.value ? compProjectUrl(project.value) : ''))
let request = 0
onScopeDispose(() => {
  request++
})
async function load() {
  const current = ++request
  loading.value = true
  error.value = ''
  project.value = undefined
  versions.value = []
  try {
    const id = String(route.query.id || '')
    if (!/^[a-zA-Z0-9_-]+$/.test(id)) throw new Error(pclText('Proto.Resource.InvalidId'))
    source.value = compSourceOf(id)
    const [data, files] = await Promise.all([getCompProject(id), getCompVersions(id)])
    if (current !== request) return
    project.value = data
    versions.value = files
    // 路由带入的是实例的具体版本号，需要归到筛选选项上
    if (
      versionFilters.value.list.length >= 2 &&
      requestedVersion.value &&
      files.some((file) => file.game_versions.includes(requestedVersion.value))
    )
      versionFilter.value = groupedVersionName(
        requestedVersion.value,
        versionFilters.value.byDrop,
        versionFilters.value.foldOld,
      )
    else versionFilter.value = ''
    loaderFilter.value =
      versionFilters.value.list.length >= 2 &&
      isMod.value &&
      loaders.value.includes(requestedLoader.value)
        ? requestedLoader.value
        : ''
  } catch (err) {
    if (current === request) error.value = pclText('Proto.Error.Resource', [String(err)])
  } finally {
    if (current === request) loading.value = false
  }
}
async function copy(value: string) {
  try {
    await navigator.clipboard.writeText(value)
    sideTip.show(pclText('Common.Hint.Copied'), 'success')
  } catch {
    sideTip.show(pclText('Proto.Clipboard.Unavailable'), 'warn')
  }
}
async function download(file: IVersion['files'][number], version: IVersion, saveOnly = false) {
  if (!project.value) return
  try {
    await saveResource(
      file,
      version,
      route.query.type === 'datapack' ? 'datapack' : project.value.project_type,
      route.query.type === 'datapack' ? datapackTargetFromQuery(route.query) : undefined,
      project.value.project_type === 'modpack' && source.value === 'modrinth' && !saveOnly
        ? project.value.title
        : undefined,
    )
  } catch (error) {
    sideTip.show(pclText('Proto.Error.Download', [String(error)]), 'warn')
  }
}
watch(
  () => [route.query.id, route.query.version, route.query.loader],
  () => {
    requestedVersion.value = route.query.version === 'all' ? '' : String(route.query.version || '')
    requestedLoader.value = String(route.query.loader || '')
    versionFilter.value = ''
    loaderFilter.value = ''
    void load()
  },
  { immediate: true },
)
</script>
<template>
  <article class="subview pcl-page detail-page">
    <PLoading v-if="loading" :loading-text="text('Download.Version.LoadingList')" />
    <PHint v-else-if="error" severity="error"
      >{{ error
      }}<PButton inline @click="load">{{
        pclText('Launch.Homepage.LoadFailed.Retry')
      }}</PButton></PHint
    >
    <template v-else-if="project && item">
      <PCard hide-title>
        <div class="intro">
          <PCompItem :data="item" />
          <div class="actions">
            <PIconTextButton bordered icon="lucide/globe" @click="() => openExternal(projectUrl)">{{
              sourceName
            }}</PIconTextButton>
            <PIconTextButton
              bordered
              icon="lucide/globe"
              @click="
                () =>
                  openExternal(
                    `https://search.mcmod.cn/s?key=${encodeURIComponent(project!.title)}`,
                  )
              "
              >{{ text('Download.Comp.Detail.McMod') }}</PIconTextButton
            >
            <PIconTextButton
              bordered
              icon="lucide/earth"
              @click="
                () =>
                  openExternal(
                    `https://translate.google.com/?sl=auto&amp;tl=zh-CN&amp;text=${encodeURIComponent(project!.description)}`,
                  )
              "
              >{{ text('Download.Comp.Detail.TranslateDescription') }}</PIconTextButton
            >
            <PIconTextButton bordered icon="lucide/copy" @click="() => copy(project!.title)">{{
              text('Download.Comp.Detail.CopyName')
            }}</PIconTextButton>
            <PIconTextButton
              bordered
              :icon="favorites.has(project.id) ? 'lucide/heart-filled' : 'lucide/heart'"
              @click="() => favorites.toggle(item!)"
              >{{
                text(
                  favorites.has(project.id)
                    ? 'Download.Comp.Detail.Unfavorite'
                    : 'Download.Comp.Detail.Favorite',
                )
              }}</PIconTextButton
            >
            <PIconTextButton bordered icon="lucide/copy" @click="() => copy(projectUrl)">{{
              text('Download.Comp.Detail.CopyLink')
            }}</PIconTextButton>
          </div>
        </div>
      </PCard>
      <PCard v-if="versionFilters.list.length >= 2" hide-title>
        <div class="filters">
          <div class="filter-row">
            <span v-if="isMod" class="filter-label">{{
              text('Download.Comp.Detail.InstanceFilter')
            }}</span>
            <button
              type="button"
              class="pcl-chip"
              :class="{ active: !versionFilter }"
              @click="versionFilter = ''"
            >
              {{ text('Common.Option.All') }}
            </button>
            <button
              v-for="value in versionFilters.list"
              :key="value"
              type="button"
              class="pcl-chip"
              :class="{ active: versionFilter === value }"
              @click="versionFilter = value"
            >
              {{ value }}
            </button>
          </div>
          <div v-if="isMod" class="filter-row">
            <span class="filter-label">{{ text('Download.Comp.Detail.LoaderFilter') }}</span>
            <button
              type="button"
              class="pcl-chip"
              :class="{ active: !loaderFilter }"
              @click="loaderFilter = ''"
            >
              {{ text('Common.Option.All') }}
            </button>
            <button
              v-for="value in loaders"
              :key="value"
              type="button"
              class="pcl-chip"
              :class="{ active: loaderFilter === value }"
              @click="loaderFilter = value"
            >
              {{ loaderNames[value] || value }}
            </button>
          </div>
        </div>
      </PCard>
      <PCard
        v-for="group in groups"
        :key="`${versionFilter}:${loaderFilter}:${group.key}`"
        :title="group.name"
        :default-fold-status="group.key === 'selected' || groups.length === 1 ? 'unfold' : 'fold'"
      >
        <div class="version-files">
          <PVirtualList
            :items="
              group.versions.flatMap((version) => version.files.map((file) => ({ version, file })))
            "
            :item-key="(row) => `${row.version.id}:${row.file.url}`"
            v-slot="{ item: { version, file } }"
          >
            <PListItem
              :title="version.name"
              :subtitle="`${file.filename} · ${version.loaders.join(', ')} · ${new Date(version.date_published).toLocaleString(pclFormatLocale)}`"
              clickable
              @click="() => download(file, version)"
            >
              <template v-if="project.project_type === 'modpack' && source === 'modrinth'" #icon-actions>
                <button
                  class="pcl-icon-button"
                  type="button"
                  :title="text('Download.Comp.Detail.DownloadModpackFile')"
                  :aria-label="text('Download.Comp.Detail.DownloadModpackFile')"
                  @click.stop="download(file, version, true)"
                >
                  <PSvgIcon name="lucide/download" :size="16" />
                </button>
              </template>
            </PListItem>
          </PVirtualList>
        </div>
      </PCard>
      <p v-if="!groups.length" class="pcl-empty">{{ pclText('Proto.Empty.FilteredVersions') }}</p>
    </template>
  </article>
</template>
<style scoped>
.detail-page {
  padding: 25px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 15px;
  height: 100%;
}
.intro {
  padding: 15px 22px;
}
.actions {
  display: flex;
  gap: 3px;
  flex-wrap: wrap;
  padding-top: 10px;
}
.filters {
  display: flex;
  flex-direction: column;
  padding: 5px 0;
}
.filter-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  margin: 5px 10px;
}
.filter-label {
  margin: 0 2px 0 2px;
  color: var(--color-text-black);
}
.version-files {
  padding: 0 20px 15px;
}
</style>
