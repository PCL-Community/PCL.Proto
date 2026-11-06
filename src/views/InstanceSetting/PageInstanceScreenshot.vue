<script setup lang="ts">
import { pclText } from '@/api/pclLocale'
import PButton from '@/components/widget/PButton.vue'
import PIconTextButton from '@/components/widget/PIconTextButton.vue'
import PCard from '@/components/widget/PCard.vue'
import { computed } from 'vue'
import { resourceUrl, useInstanceResources } from '@/composables/useInstanceResources'
import { openInstancePath } from '@/api/instanceResources'
import { useSelectedInstance } from '@/stores/gameLaunch'
import sideTip from '@/composables/sideTip'
import PLoading from '@/components/widget/PLoading.vue'
import PHint from '@/components/widget/PHint.vue'

const instance = useSelectedInstance()

const resources = useInstanceResources({
  kind: () => 'screenshot',
  messages: {
    deleted: 'Instance.Screenshot.Deleted',
    deletedMultiple: 'Instance.Resource.Delete.RecycleMultiple',
    deleteFailed: 'Instance.Screenshot.DeleteFailed',
  },
})
const screenshots = computed(() =>
  resources.items.value.map((entry) => ({
    id: entry.path,
    url: resourceUrl(entry.token),
    name: entry.name,
  })),
)

function remove(id: string) {
  const entry = resources.items.value.find((item) => item.path === id)
  if (entry) void resources.remove([entry])
}

async function openScreenshot(id: string) {
  const directory = instance.instance_info?.directory
  if (!directory) return
  try {
    await openInstancePath(directory, id)
  } catch (failure) {
    sideTip.show(`${pclText('Instance.Screenshot.OpenFailed')} ${failure}`, 'warn')
  }
}

</script>
<template>
  <div class="pcl-page">
    <PLoading
      v-if="resources.loading.value"
      :loading-text="pclText('Instance.Saves.Status.Refreshing')"
    />
    <PHint v-else-if="resources.error.value" severity="error">{{ resources.error.value }}</PHint>
    <div v-else-if="screenshots.length" class="screenshots-content">
      <PCard :title="pclText('Instance.Screenshot.QuickActions')"
        ><div class="screenshot-actions">
          <PButton type="tint" @click="resources.openFolder()">{{
            pclText('Instance.Screenshot.OpenFolder')
          }}</PButton>
        </div></PCard
      >
      <div class="screenshots">
        <PCard
          v-for="screenshot in screenshots"
          :key="screenshot.id"
          hide-title
          class="screenshot-card"
          :title="screenshot.name"
        >
          <button
            class="screenshot-preview"
            type="button"
            :aria-label="screenshot.name"
            @click="openScreenshot(screenshot.id)"
          >
            <img :src="screenshot.url" :alt="screenshot.name" loading="lazy" />
          </button>
          <div class="screenshot-card-actions">
            <PIconTextButton icon="lucide/folder-open" @click="openScreenshot(screenshot.id)">{{
              pclText('Common.Action.Open')
            }}</PIconTextButton>
            <PIconTextButton icon="lucide/trash-2" @click="remove(screenshot.id)">{{
              pclText('Common.Action.Delete')
            }}</PIconTextButton>
          </div>
        </PCard>
      </div>
    </div>
    <PCard v-else class="pcl-empty-state-card" hide-title>
      <div class="pcl-empty-state">
        <span class="pcl-empty-state-title pcl-text">{{
          pclText('Instance.Screenshot.Empty.Title')
        }}</span>
        <div class="pcl-divider" />
        <span class="pcl-empty-state-message pcl-text">{{
          pclText('Instance.Screenshot.Empty.Message')
        }}</span>
        <div class="pcl-empty-state-actions">
          <PButton type="tint" @click="resources.openFolder()">{{
            pclText('Instance.Screenshot.OpenFolder')
          }}</PButton>
        </div>
      </div>
    </PCard>
  </div>
</template>

<style scoped>
.screenshots-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.screenshot-actions {
  padding: 0 25px 20px;
}
.screenshot-actions :deep(button) {
  min-width: 140px;
  min-height: 35px;
}
.screenshots {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 14px;
  padding: 0 8px 25px;
}
.screenshot-card {
  width: 220px;
}
.screenshot-preview {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 129px;
  padding: 9px 8px 0;
  border: 0;
  background: transparent;
  cursor: pointer;
}
.screenshot-card-actions {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 12px;
  padding: 5px 3px;
}
.screenshots img {
  max-width: 100%;
  max-height: 120px;
  object-fit: contain;
}
</style>
