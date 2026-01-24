<script lang="ts">
type ProjectType = 'mod' | 'modpack' | 'resourcepack' | 'shader' | 'world'

const apiSupports: Record<'modrinth' | 'curseforge', ProjectType[]> = {
  modrinth: ['mod', 'modpack', 'resourcepack', 'shader'],
  curseforge: ['mod', 'modpack', 'resourcepack', 'shader', 'world'],
}
</script>

<script setup lang="ts">
import modrinthApi, { type ISearchHit } from '@/api/modrinthApi'
import SetupItem from '@/components/widget/SetupItem.vue'
import PCard from '@/components/widget/PCard.vue'
import PCompItem from '@/components/widget/PCompItem.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PSearchBox from '@/components/widget/PSearchBox.vue'
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'

const hits = ref<ISearchHit[] | null | undefined>(undefined)
const route = useRoute()
const { t } = useI18n()

// [TODO] 支持更多API平台如Curseforge
const handleSearch = async (query?: string) => {
  hits.value = undefined
  let data = await modrinthApi.searchProjects({
    query,
    facets: [['project_type:' + route.meta.project_type]],
  })
  hits.value = data.hits
}

const placeholders: Record<ProjectType, string> = {
  mod: t('download.search_for') + ' ' + t('download.nav.mod'),
  modpack: t('download.search_for') + ' ' + t('download.nav.mod_pack'),
  resourcepack: t('download.search_for') + ' ' + t('download.nav.resource_pack'),
  shader: t('download.search_for') + ' ' + t('download.nav.shader_pack'),
  world: t('download.search_for') + ' ' + t('download.nav.world'),
}

watch(
  () => route.meta.project_type,
  (newType, oldType) => {
    if (newType !== oldType) {
      handleSearch()
    }
  },
  { immediate: true },
)
</script>

<template>
  <PSearchBox
    @search="handleSearch"
    :placeholder="placeholders[route.meta.project_type as ProjectType]"
  />
  <PCard hide-title>
    <div class="page-comp-options">
      <SetupItem label="来源" type="select" :options="[{ key: 'all', text: '全部' }, { key: 'modrinth', text: 'Modrinth' }, { key: 'curseforge', text: 'CurseForge' }]"></SetupItem>
      <SetupItem label="标签" type="select" :options="[{ key: 'all', text: '全部' }]"></SetupItem>
      <SetupItem
        label="排序方式"
        type="select"
        :options="[
          { key: 'default', text: '默认' },
          { key: 'relevance', text: '相关性' },
          { key: 'downloads', text: '下载量' },
          { key: 'followers', text: '关注量' },
          { key: 'newest', text: '最新发布' },
          { key: 'updated', text: '最近更新' },
        ]"
      ></SetupItem>
    </div>
    <div class="page-comp-options">
      <SetupItem label="版本" type="select" :options="[{ key: 'any', text: '任意' }]"></SetupItem>
      <SetupItem
        label="加载器"
        type="select"
        :options="[
          { key: 'any', text: '任意' },
          { key: 'forge', text: 'Forge' },
          { key: 'neoforge', text: 'NeoForge' },
          { key: 'fabric', text: 'Fabric' },
          { key: 'quilt', text: 'Quilt' },
        ]"
      ></SetupItem>
    </div>
  </PCard>
  <PCard hide-title v-if="hits && hits.length > 0" ref="projectCard" v-card-drop-animate>
    <PCompItem v-for="project in hits" :data="project" clickable />
  </PCard>
  <div class="loading-page" v-else-if="!hits">
    <PLoading :state="hits === null ? 'error' : 'loading'" />
  </div>
</template>

<style scoped lang="scss">
.page-comp-options {
  display: flex;
  gap: 1rem;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;

  :deep(.setupitem-with-text-container) {
    &:nth-child(1) {
      flex: 2;
    }
    &:nth-child(2) {
      flex: 2;
    }
    &:nth-child(3) {
      flex: 4;
    }
  }
}
</style>
