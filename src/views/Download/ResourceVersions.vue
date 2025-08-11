<script setup lang="ts">
import modrinthApi, { type IProject } from '@/api/modrinthApi'
import PCard from '@/components/widget/PCard.vue'
import PCompItem from '@/components/widget/PCompItem.vue'
import PLoading from '@/components/widget/PLoading.vue'
import { marked } from 'marked'
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const projectData = ref<IProject>()

onMounted(async () => {
  projectData.value = await modrinthApi.getProject(route.query.id as string)
})
</script>

<template>
  <article class="subview" v-if="projectData">
    <PCard hide-title>
      <PCompItem
        :title="projectData.title"
        :categories="projectData.categories"
        :downloads="projectData.downloads"
        :icon_url="projectData.icon_url"
        :description="projectData.description"
        :date_modified="projectData.updated"
        :project_id="projectData.id"
        :project_type="projectData.project_type"
      />
    </PCard>
    <PCard title="资源简介" default-fold-status="fold">
      <div class="markdown" v-html="marked.parse(projectData.body)" />
    </PCard>
    <PCard hide-title>
      <menu>
        <li class="active">全部</li>
        <li>1.21</li>
        <li>1.20</li>
        <li>1.19</li>
        <li>1.18</li>
        <li>快照版本</li>
      </menu>
    </PCard>
  </article>
  <div v-else class="loading-page">
    <PLoading state="loading" />
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
}

:deep(.markdown strong) {
  font-weight: bold;
}

:deep(.markdown img) {
  height: auto;
}
</style>
