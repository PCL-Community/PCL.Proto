<script setup lang="ts">
import { pclText } from '@/api/pclLocale'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import PListItem from '@/components/widget/PListItem.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PSearchBox from '@/components/widget/PSearchBox.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import { resourceUrl, useInstanceResources } from '@/composables/useInstanceResources'
import { openInstancePath } from '@/api/instanceResources'
import { useSelectedInstance } from '@/stores/gameLaunch'
import { isTauri } from '@tauri-apps/api/core'
import sideTip from '@/composables/sideTip'

const router = useRouter()
const instance = useSelectedInstance()
const sort = ref('name')
const query = ref('')

const resources = useInstanceResources({
  kind: () => 'save',
  messages: {
    deleted: 'Instance.Saves.DeletedToRecycleBin',
    deletedMultiple: 'Instance.Resource.Delete.RecycleMultiple',
    deleteFailed: 'Instance.Saves.DeleteFailed',
  },
})

function formatDate(seconds: number) {
  return seconds ? new Date(seconds * 1000).toLocaleString() : ''
}

const saves = computed(() =>
  resources.items.value.map((entry) => ({
    id: entry.path,
    name: entry.name,
    description: formatDate(entry.modified),
    icon: resourceUrl(entry.token),
  })),
)
const matching = computed(() =>
  saves.value.filter((save) =>
    `${save.name} ${save.description || ''}`.toLowerCase().includes(query.value.toLowerCase()),
  ),
)
const savesSorted = computed(() =>
  sort.value === 'name'
    ? [...matching.value].sort((a, b) => a.name.localeCompare(b.name))
    : matching.value,
)

function removeSave(id: string) {
  const entry = resources.items.value.find((item) => item.path === id)
  if (entry) void resources.remove([entry])
}

async function openSave(id: string) {
  const directory = instance.instance_info?.directory
  if (!directory || !isTauri()) return
  try {
    await openInstancePath(directory, id)
  } catch (failure) {
    sideTip.show(`${text('Instance.Saves.Error.OperationFailed')} ${failure}`, 'warn')
  }
}

const text = pclText
</script>
<template>
  <div class="pcl-page">
    <div>
      <div data-control="PanBack">
        <div style="display: grid">
          <div v-if="saves.length" class="saves-list">
            <PSearchBox v-model="query" :placeholder="text('Instance.Saves.SearchHint')" />
            <PCard :title="text('Instance.Saves.QuickActions')"
              ><div class="saves-actions">
                <PButton @click="resources.openFolder()">{{
                  text('Instance.Saves.OpenFolder')
                }}</PButton
                ><PButton
                  @click="
                    () => router.push({ path: '/download/world', query: { instance: 'selected' } })
                  "
                  >{{ text('Instance.Resource.DownloadNew') }}</PButton
                ><PButton @click="resources.importFiles()">{{
                  text('Instance.Resource.InstallFromFiles')
                }}</PButton>
              </div></PCard
            >
            <PCard :title="text('Instance.Saves.ListTitle')"
              ><div class="saves-actions">
                <label>{{ text('Instance.Saves.SortBy') }}</label
                ><PComboBox
                  v-model="sort"
                  :options="[
                    { key: 'name', text: pclText('Proto.Label.Name') },
                    { key: 'default', text: text('Common.Option.Default') },
                  ]"
                />
              </div>
              <div class="save-items">
                <PListItem
                  v-for="save in savesSorted"
                  :key="save.id"
                  :title="save.name"
                  :subtitle="save.description"
                  :icon="save.icon"
                  round-img
                  clickable
                  @click="() => openSave(save.id)"
                >
                  <template #actions>
                    <button
                      class="pcl-icon-button"
                      type="button"
                      :title="text('Common.Action.Delete')"
                      :aria-label="text('Common.Action.Delete')"
                      @click="removeSave(save.id)"
                    >
                      <PSvgIcon name="lucide/trash-2" :size="15" />
                    </button>
                  </template>
                </PListItem></div
            ></PCard>
          </div>
          <PCard
            v-if="!saves.length"
            data-control="PanNoWorld"
            style="
              margin: 40px;
              grid-row: 1 / span 1;
              grid-column: 1 / span 1;
              justify-self: center;
              align-self: center;
            "
            hide-title
          >
            <div
              style="
                margin: 17px 20px;
                display: grid;
                grid-template-columns: minmax(0, 1fr) auto auto auto minmax(0, 1fr);
                grid-template-rows: auto auto auto auto;
              "
            >
              <span
                style="
                  margin: 0px 0px 9px 0px;
                  font-size: 19px;
                  grid-row: 1 / span 1;
                  grid-column: 1 / span 4;
                  justify-self: center;
                  color: var(--color-tint);
                "
                class="pcl-text"
                >{{ text('Instance.Saves.Empty.Title') }}</span
              >
              <div
                style="
                  height: 2px;
                  grid-row: 2 / span 1;
                  grid-column: 1 / span 4;
                  justify-self: stretch;
                "
                class="pcl-divider"
              />
              <span
                style="
                  margin: 15px 10px 5px 10px;
                  grid-row: 3 / span 1;
                  grid-column: 1 / span 4;
                  justify-self: center;
                "
                class="pcl-text"
                >{{ text('Instance.Saves.Empty.Message') }}</span
              >
              <PButton
                style="
                  height: 35px;
                  min-width: 140px;
                  margin: 10px 10px 0px 10px;
                  padding: 0px 13px;
                  grid-row: 4 / span 1;
                  grid-column: 2 / span 1;
                  justify-self: center;
                "
                @click="resources.openFolder()"
                type="tint"
                >{{ text('Instance.Saves.OpenFolder') }}</PButton
              >
              <PButton
                style="
                  height: 35px;
                  min-width: 140px;
                  margin: 10px 10px 0px 10px;
                  padding: 0px 13px;
                  grid-row: 4 / span 1;
                  grid-column: 3 / span 1;
                  justify-self: center;
                "
                @click="
                  () => router.push({ path: '/download/world', query: { instance: 'selected' } })
                "
                >{{ text('Instance.Resource.DownloadNew') }}</PButton
              >
              <PButton
                style="
                  height: 35px;
                  min-width: 140px;
                  margin: 10px 10px 0px 10px;
                  padding: 0px 13px;
                  grid-row: 4 / span 1;
                  grid-column: 4 / span 1;
                  justify-self: center;
                "
                @click="resources.importFiles()"
                >{{ text('Instance.Resource.InstallFromFiles') }}</PButton
              >
            </div>
          </PCard>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.saves-list {
  padding: 25px;
  display: flex;
  flex-direction: column;
  gap: 15px;
}
.saves-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 15px;
  padding: 0 25px 15px;
}
.save-items {
  padding: 0 20px 20px;
}
</style>
