<script setup lang="ts">
import modrinthApi, { type ISearchHit } from '@/api/modrinthApi'
import PCompItem from '@/components/widget/PCompItem.vue'
import PCard from '@/components/widget/PCard.vue'
import PVirtualList from '@/components/widget/PVirtualList.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PButton from '@/components/widget/PButton.vue'
import PTextBox from '@/components/widget/PTextBox.vue'
import PSearchBox from '@/components/widget/PSearchBox.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import PHint from '@/components/widget/PHint.vue'
import PLoading from '@/components/widget/PLoading.vue'
import { compFacets, compTagOptions } from '@/api/compResources'
import { computed, reactive, ref, watch } from 'vue'
import { pclText } from '@/api/pclLocale'

const props = defineProps<{
  projectType: 'mod' | 'modpack' | 'resourcepack' | 'shader' | 'datapack' | 'world'
}>()
const text = pclText
const searchHint = computed(() =>
  text('Proto.Resource.Search', [
    text(
      `Download.Comp.Type.${{ mod: 'Mod', modpack: 'Modpack', resourcepack: 'ResourcePack', shader: 'Shader', datapack: 'DataPack', world: 'World' }[props.projectType]}`,
    ),
  ]),
)
const values = reactive({ source: '0', sort: '0', loader: '', version: '', query: '', tag: '' })
const hits = ref<ISearchHit[]>([])
const error = ref('')
const loading = ref(false)
const page = ref(0)
const total = ref(0)
let request = 0
const limit = 40
const tagOptions = computed(() => compTagOptions(props.projectType))
const loaderOptions = computed(() =>
  (props.projectType === 'shader'
    ? ['', 'vanilla', 'iris', 'optifine']
    : ['', 'forge', 'neoforge', 'fabric', 'quilt']
  ).map((value) => ({
    key: value,
    text:
      value === 'vanilla'
        ? text('Download.Comp.Filter.Loader.VanillaAvailable')
        : value
          ? text(
              `Common.Installation.${({ forge: 'Forge', neoforge: 'NeoForge', fabric: 'Fabric', quilt: 'Quilt', iris: 'Iris', optifine: 'OptiFine' } as Record<string, string>)[value]}`,
            )
          : text('Download.Comp.Filter.Loader.Any'),
  })),
)
async function search() {
  const current = ++request
  error.value = ''
  loading.value = true
  try {
    if (values.source === '1') throw new Error(pclText('Proto.Resource.CurseForgeUnavailable'))
    if (props.projectType === 'world') throw new Error(pclText('Proto.Resource.WorldUnavailable'))
    const sort =
      (['relevance', 'downloads', 'follows', 'newest', 'updated'] as const)[
        Number(values.sort) - 1
      ] || (values.query.trim() ? 'relevance' : 'downloads')
    const data = await modrinthApi.searchProjects({
      query: values.query,
      facets: compFacets(props.projectType, values),
      index: sort,
      offset: page.value * limit,
      limit,
    })
    if (current !== request) return
    hits.value = data.hits
    total.value = data.total_hits
  } catch (err) {
    if (current !== request) return
    hits.value = []
    total.value = 0
    error.value =
      err instanceof Error ? err.message : pclText('Proto.Error.Resources', [String(err)])
  } finally {
    if (current === request) loading.value = false
  }
}
function reset() {
  Object.assign(values, { source: '0', sort: '0', loader: '', version: '', query: '', tag: '' })
  page.value = 0
  void search()
}
function firstSearch() {
  page.value = 0
  void search()
}
function turn(next: number) {
  page.value = Math.max(0, next)
  void search()
}
watch(() => props.projectType, reset, { immediate: true })
</script>
<template>
  <section class="pcl-page comp-page">
    <PSearchBox
      v-model="values.query"
      search-button
      :placeholder="searchHint"
      @search="firstSearch"
    />
    <PCard hide-title
      ><div class="filters">
        <label for="comp-source">{{ text('Download.Comp.Filter.Source') }}</label>
        <PComboBox
          v-model="values.source"
          input-id="comp-source"
          :options="[
            { key: '0', text: 'Modrinth' },
            {
              key: '1',
              text: 'CurseForge',
              disabled: true,
              title: text('Proto.Resource.CurseForgeUnavailable'),
            },
          ]"
          @update:model-value="firstSearch"
        />
        <label for="comp-tag">{{ text('Download.Comp.Filter.Tag') }}</label>
        <PComboBox
          input-id="comp-tag"
          v-model="values.tag"
          :options="tagOptions"
          @update:model-value="firstSearch"
        />
        <label for="comp-sort">{{ text('Download.Comp.Filter.Sort') }}</label>
        <PComboBox
          v-model="values.sort"
          input-id="comp-sort"
          :options="[
            { key: '0', text: text('Common.Option.Default') },
            ...['Relevance', 'Downloads', 'Follows', 'Latest', 'Updated'].map((item, index) => ({
              key: String(index + 1),
              text: text(`Download.Comp.Sort.${item}`),
            })),
          ]"
          @update:model-value="firstSearch"
        />
        <button
          type="button"
          class="pcl-icon-button reset"
          @click="reset"
          :title="text('Download.Comp.Filter.Reset')"
          :aria-label="text('Download.Comp.Filter.Reset')"
        >
          <PSvgIcon name="lucide/rotate-ccw" :size="16" />
        </button>
        <label for="comp-version">{{ text('Download.Comp.Filter.Version') }}</label>
        <PTextBox
          v-model="values.version"
          input-id="comp-version"
          :placeholder="text('Download.Comp.Filter.Version.Any')"
          :suggestions="[
            '1.21.11',
            '1.21.1',
            '1.20.6',
            '1.20.1',
            '1.19.4',
            '1.19.2',
            '1.18.2',
            '1.16.5',
            '1.12.2',
            '1.10.2',
            '1.8.9',
            '1.7.10',
          ]"
          @change="firstSearch"
          @keydown.enter="firstSearch"
        />
        <template v-if="['mod', 'modpack', 'shader'].includes(projectType)">
          <label for="comp-loader">{{ text('Download.Comp.Filter.Loader') }}</label>
          <PComboBox
            v-model="values.loader"
            input-id="comp-loader"
            :options="loaderOptions"
            @update:model-value="firstSearch"
          />
        </template></div
    ></PCard>
    <PLoading v-if="loading" :loading-text="text('Download.Version.LoadingList')" />
    <PHint v-else-if="error" severity="error"
      >{{ error }}
      <PButton inline @click="search">{{
        pclText('Launch.Homepage.LoadFailed.Retry')
      }}</PButton></PHint
    >
    <template v-else>
      <PCard hide-title
        ><div class="projects">
          <PVirtualList
            :items="hits"
            :item-key="(project) => project.project_id"
            :row-height="64"
            v-slot="{ item: project }"
          >
            <PCompItem
              :data="project"
              :game-version="values.version.trim()"
              :loader="projectType === 'datapack' ? 'datapack' : values.loader"
              clickable
            />
          </PVirtualList>
          <p v-if="!hits.length" class="pcl-empty">{{ pclText('Proto.Empty.Resources') }}</p>
        </div></PCard
      >
      <PCard hide-title class="pages-card"
        ><nav class="pages" :aria-label="pclText('Proto.Label.Pagination')">
          <button
            type="button"
            class="pcl-icon-button"
            :disabled="!page"
            @click="() => turn(0)"
            :aria-label="pclText('Proto.Action.First')"
          >
            <PSvgIcon name="lucide/chevrons-left" :size="14" />
          </button>
          <button
            type="button"
            class="pcl-icon-button"
            :disabled="!page"
            @click="() => turn(page - 1)"
            :aria-label="pclText('Proto.Action.Previous')"
          >
            <PSvgIcon name="lucide/chevron-left" :size="14" />
          </button>
          <span>{{ page + 1 }}</span>
          <button
            type="button"
            class="pcl-icon-button"
            :disabled="(page + 1) * limit >= total"
            @click="() => turn(page + 1)"
            :aria-label="pclText('Proto.Action.Next')"
          >
            <PSvgIcon name="lucide/chevron-right" :size="14" />
          </button></nav
      ></PCard>
    </template>
  </section>
</template>
<style scoped>
.comp-page {
  padding: 25px;
  display: flex;
  flex-direction: column;
  gap: 15px;
}
.filters {
  display: grid;
  grid-template-columns: auto minmax(65px, 1fr) auto minmax(80px, 1.3fr) auto minmax(
      65px,
      1fr
    ) 22px;
  grid-template-rows: 28px 28px;
  align-items: center;
  gap: 12px 8px;
  padding: 12px;
}
.filters label {
  white-space: nowrap;
}
.filters label[for='comp-tag'],
.filters label[for='comp-sort'],
.filters label[for='comp-loader'] {
  margin-inline-start: 6px;
}
.filters .reset {
  min-height: 0;
  width: 22px;
  height: 22px;
  padding: 3px;
}
.projects {
  padding: 12px;
}
.pages-card {
  align-self: center;
}
.pages {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  padding: 7px 10px;
}
.pages span {
  margin: 0 8px;
  font-size: 15px;
  height: 23px;
  line-height: 23px;
  color: var(--color-tint);
}
.pages .pcl-icon-button {
  width: 23px;
  height: 23px;
  padding: 4px;
}
.pages .pcl-icon-button:disabled {
  opacity: 0.2;
}
@media (max-width: 700px) {
  .filters {
    grid-template-columns: auto minmax(70px, 1fr) auto minmax(90px, 1.3fr);
    grid-template-rows: none;
  }
  .reset {
    grid-column: 4;
    grid-row: 2;
    justify-self: end;
  }
}
</style>
