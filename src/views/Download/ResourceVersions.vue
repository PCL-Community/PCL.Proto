<script setup lang="ts">
import modrinthApi, { type IProject } from '@/api/modrinthApi'
import PCard from '@/components/widget/PCard.vue'
import PCompItem from '@/components/widget/PCompItem.vue'
import PLoading from '@/components/widget/PLoading.vue'
import { useRouteQuery } from '@vueuse/router'
import { marked } from 'marked'
import { computed, onMounted, ref, type Ref } from 'vue'
import ResouceVersionsList from './ResouceVersionsList.vue'

const project_id = useRouteQuery('id') as Ref<string>
const version = useRouteQuery('version') as Ref<string>
const projectData = ref<IProject>()

const data = computed(() => {
  if (!projectData.value) return undefined
  return {
    ...projectData.value,
    date_modified: projectData.value.updated,
    project_id: project_id.value,
    body: marked.parse(projectData.value.body),
  }
})

const specificVersions = ['1.21', '1.20', '1.19', '1.18', '1.17', '1.16', '1.15', '1.14']

const switchVersion = (v: string) => {
  version.value = v
}

onMounted(async () => {
  projectData.value = await modrinthApi.getProject(project_id.value)
})
</script>

<template>
  <article class="subview" v-if="data" v-card-drop-children-animate>
    <PCard hide-title>
      <PCompItem :data="data" :clickable="false" />
    </PCard>
    <PCard title="资源简介" default-fold-status="fold">
      <article class="markdown" v-html="data.body" />
    </PCard>
    <PCard hide-title>
      <menu>
        <li :class="{ active: version === 'all' }" @click="switchVersion('all')">全部</li>
        <li
          v-for="v in specificVersions"
          :key="v"
          :class="{ active: version === v }"
          @click="switchVersion(v)"
        >
          {{ v }}
        </li>
        <li :class="{ active: version === 'snapshot' }" @click="switchVersion('snapshot')">
          快照版本
        </li>
        <li :class="{ active: version === 'old' }" @click="switchVersion('old')">远古版本</li>
      </menu>
    </PCard>
    <ResouceVersionsList v-card-drop-animate />
  </article>
  <div v-else class="loading-page" v-card-drop-animate>
    <PLoading state="loading" loading-text="正在加载资源信息……" />
  </div>
</template>

<style lang="css" scoped>
.subview {
  height: 100%;
  overflow-y: scroll;
}

menu {
  display: flex;
  list-style-type: none;
  padding: 0;
  justify-content: flex-start;
  gap: 8px;
  flex-wrap: wrap;
}

menu > li {
  padding: 4px 8px;
  border-radius: 1rem;
  transition: background-color 0.4s ease;
  user-select: none;
  color: var(--color-tint);
}

menu > li:hover {
  background-color: var(--color-tint-lighter);
}

menu > li.active {
  background-color: var(--color-tint);
  color: white;
}

.markdown {
  font-family:
    'HarmonyOS Sans SC',
    Inter,
    -apple-system,
    BlinkMacSystemFont,
    'Segoe UI',
    Roboto,
    Oxygen,
    Ubuntu,
    Cantarell,
    'Fira Sans',
    'Droid Sans',
    'Helvetica Neue',
    sans-serif;
}

:deep(.markdown *) {
  max-width: 100%;
  user-select: text;
  -webkit-user-select: text;
}

:deep(.markdown strong) {
  font-weight: bold;
}

:deep(.markdown img) {
  height: auto;
}
</style>
