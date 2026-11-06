<script setup lang="ts">
import { useRouter } from 'vue-router'
import { pclText } from '@/api/pclLocale'
import { computed, ref } from 'vue'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PListItem from '@/components/widget/PListItem.vue'
import PSearchBox from '@/components/widget/PSearchBox.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PHint from '@/components/widget/PHint.vue'
type Entry = { id: string; title: string; description?: string }
const props = withDefaults(
  defineProps<{
    relations?: Entry[]
    bundled?: Entry[]
    warnings?: Entry[]
    conflicts?: Entry[]
    loading?: boolean
    error?: string
  }>(),
  {
    relations: () => [],
    bundled: () => [],
    warnings: () => [],
    conflicts: () => [],
    loading: false,
    error: '',
  },
)
const emit = defineEmits<{ refresh: [] }>()

const query = ref('')
const groups = computed(() =>
  [
    { key: 'Relations', entries: props.relations },
    { key: 'Bundled', entries: props.bundled },
  ].map((group) => ({
    ...group,
    entries: group.entries.filter((entry) =>
      `${entry.title} ${entry.description || ''}`.toLowerCase().includes(query.value.toLowerCase()),
    ),
  })),
)

const router = useRouter()
</script>
<template>
  <section class="pcl-page">
    <PLoading v-if="loading" :loading-text="pclText('Instance.Resource.Loading')" />
    <PHint v-else-if="error" severity="error"
      >{{ error
      }}<PButton inline @click="() => emit('refresh')">{{
        pclText('Common.Action.Refresh')
      }}</PButton></PHint
    >
    <template v-else-if="relations.length || bundled.length || warnings.length || conflicts.length">
      <PCard hide-title
        ><div class="manage">
          <PButton type="tint" @click="() => router.back()">{{
            pclText('Instance.Resource.BackToParent')
          }}</PButton>
          <PButton @click="() => emit('refresh')">{{ pclText('Common.Action.Refresh') }}</PButton>
          <span>{{ pclText('Instance.Resource.Mod.JarInJar.Message') }}</span>
        </div></PCard
      >
      <PSearchBox
        :placeholder="pclText('Instance.Resource.Mod.JarInJar.Search.Hint')"
        @search="(value: string) => (query = value || '')"
      />
      <PCard v-if="warnings.length" :title="pclText('Instance.Resource.Mod.JarInJar.Section.Warnings')"
        ><div class="issues">
          <PHint v-for="entry in warnings" :key="entry.id" severity="warning"
            >{{ entry.title }}
            <p v-if="entry.description">{{ entry.description }}</p></PHint
          >
        </div></PCard
      >
      <PCard
        v-if="conflicts.length"
        :title="pclText('Instance.Resource.Mod.JarInJar.Section.Conflicts')"
        ><div class="issues" :title="pclText('Instance.Resource.Mod.JarInJar.Conflict.Tooltip')">
          <PListItem
            v-for="entry in conflicts"
            :key="entry.id"
            :title="entry.title"
            :subtitle="entry.description"
          /></div
      ></PCard>
      <template v-for="group in groups" :key="group.key"
        ><section v-if="group.entries.length">
          <h2>{{ pclText(`Instance.Resource.Mod.JarInJar.Section.${group.key}`) }}</h2>
          <PListItem
            v-for="entry in group.entries"
            :key="entry.id"
            :title="entry.title"
            :subtitle="entry.description"
          /></section
      ></template>
      <p v-if="query && groups.every((group) => !group.entries.length)" class="pcl-empty">
        {{ pclText('Download.Comp.List.NoMatchingResults') }}
      </p>
    </template>
    <PCard v-else hide-title class="pcl-empty-state-card"
      ><div class="pcl-empty-state">
        <h2 class="pcl-empty-state-title">
          {{ pclText('Instance.Resource.Mod.JarInJar.Empty.Title') }}
        </h2>
        <div class="pcl-divider" />
        <p class="pcl-empty-state-message">
          {{ pclText('Instance.Resource.Mod.JarInJar.Empty.Message') }}
        </p>
        <div class="pcl-empty-state-actions">
          <PButton type="tint" @click="() => router.back()">{{
            pclText('Instance.Resource.BackToParent')
          }}</PButton>
        </div>
      </div></PCard
    >
  </section>
</template>
<style scoped>
section > h2 {
  font-size: 16px;
  color: var(--color-tint);
}
.manage {
  padding: 15px;
  display: flex;
  align-items: center;
  gap: 15px;
  flex-wrap: wrap;
}
.issues {
  padding: 0 15px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
</style>
