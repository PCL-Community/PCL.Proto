<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'
import { showUnavailable } from '@/util/showUnavailable'
import { pclText } from '@/api/pclLocale'
import { usePageSettings } from '@/composables/usePageSettings'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PSearchBox from '@/components/widget/PSearchBox.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import { computed, ref, watch } from 'vue'
import PListItem from '@/components/widget/PListItem.vue'
import PCheckBox from '@/components/widget/PCheckBox.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import { useInstanceResources } from '@/composables/useInstanceResources'

const route = useRoute()
const router = useRouter()
const { values } = usePageSettings('PageInstanceSavesDatapack', {
  filter: 'All',
  SearchBox: '',
})
const selectedSave = ref(
  String(route.query.id || '')
    .split(/[\\/]/)
    .pop() || '',
)
const saveList = useInstanceResources({ kind: () => 'save' })
const saveOptions = computed(() =>
  saveList.items.value.map((entry) => ({ key: entry.name, text: entry.name })),
)
watch(saveOptions, (options) => {
  if (!selectedSave.value && options.length) selectedSave.value = options[0]?.key || ''
})

const resources = useInstanceResources({
  kind: () => 'datapack',
  save: () => selectedSave.value || undefined,
  messages: {
    deleted: 'Instance.Saves.Datapack.Delete.RecycleSingle',
    deletedMultiple: 'Instance.Saves.Datapack.Delete.RecycleMultiple',
    deleteFailed: 'Instance.Saves.Datapack.Delete.FileOccupied',
    toggleFailed: 'Instance.Saves.Datapack.ToggleWarning',
  },
})
const loading = computed(() => saveList.loading.value || resources.loading.value)
const datapacks = computed(() =>
  resources.items.value.map((entry) => ({
    id: entry.path,
    name: entry.name,
    description: entry.size ? `${Math.round(entry.size / 1024)} KB` : '',
    enabled: entry.enabled,
    updatable: false,
    error: false,
  })),
)

const selected = ref(new Set<string>())
function select(id: string, checked: boolean) {
  if (checked) selected.value.add(id)
  else selected.value.delete(id)
}
function toggleItem(id: string) {
  const entry = resources.items.value.find((item) => item.path === id)
  if (entry) void resources.toggle(entry)
}
function runSelection(action: string) {
  const chosen = resources.items.value.filter((entry) => selected.value.has(entry.path))
  if (action === 'Enable') void resources.setEnabled(chosen, true)
  else if (action === 'Disable') void resources.setEnabled(chosen, false)
  else if (action === 'Delete') void resources.remove(chosen)
  else {
    showUnavailable()
    return
  }
  selected.value.clear()
}
const selectionActions = [
  { id: 'Update', key: 'Instance.Resource.Update', icon: 'upload' },
  { id: 'Enable', key: 'Instance.Resource.Enable', icon: 'circle-check' },
  { id: 'Disable', key: 'Instance.Resource.Disable', icon: 'circle-minus' },
  { id: 'Favorites', key: 'Instance.Resource.Favorite', icon: 'heart' },
  { id: 'Share', key: 'Instance.Resource.ShareSelected', icon: 'share-2' },
  { id: 'Delete', key: 'Common.Action.Delete', icon: 'trash-2' },
]
const visibleDatapacks = computed(() =>
  datapacks.value.filter((pack) => {
    const matches = `${pack.name} ${pack.description || ''}`
      .toLowerCase()
      .includes(String(values.SearchBox).toLowerCase())
    const filter = values.filter
    return (
      matches &&
      (filter === 'All' ||
        (filter === 'Enabled' && pack.enabled) ||
        (filter === 'Disabled' && !pack.enabled))
    )
  }),
)

const text = pclText
</script>
<template>
  <div class="pcl-page datapack-page">
    <PLoading
      v-if="loading"
      data-control="PanLoad"
      :loading-text="text('Instance.Resource.Datapack.Loading')"
    />
    <div v-else>
      <div style="display: grid">
        <div
          data-control="PanAllBack"
          style="grid-row: 1 / span 1; grid-column: 1 / span 1; display: grid"
        >
          <div data-control="PanBack" style="grid-row: 1 / span 1; grid-column: 1 / span 1">
            <PCard
              data-control="PanSaveSelect"
              style="margin: 15px 25px 0px 25px"
              :title="text('Instance.Saves.Info.LevelName')"
            >
              <div style="margin: 0px 15px 15px 15px">
                <PComboBox v-model="selectedSave" :options="saveOptions" />
              </div>
            </PCard>
            <div
              v-if="datapacks.length"
              data-control="PanMain"
              style="margin: 10px 25px 10px 25px; display: flex; flex-direction: column"
            >
              <PSearchBox
                data-control="SearchBox"
                style="margin: 15px 0px"
                :placeholder="text('Instance.Resource.Search.Hint')"
                @search="(query: string) => (values.SearchBox = query || '')"
              />
              <PCard data-control="PanManage" style="margin: 0px 0px 15px 0px" hide-title>
                <div
                  style="
                    margin: 8px 0px 15px 15px;
                    display: flex;
                    flex-direction: row;
                    align-items: center;
                    flex-wrap: wrap;
                  "
                >
                  <PButton
                    data-control="BtnManageOpen"
                    style="min-width: 110px; margin: 7px 15px 0px 0px; padding: 7px 13px"
                    @click="resources.openFolder()"
                    type="tint"
                    >{{ text('Common.Action.OpenFolder') }}</PButton
                  >
                  <PButton
                    data-control="BtnManageInstall"
                    style="min-width: 110px; margin: 7px 15px 0px 0px; padding: 7px 13px"
                    @click="resources.importFiles()"
                    >{{ text('Instance.Resource.InstallFromFiles') }}</PButton
                  >
                  <PButton
                    data-control="BtnManageDownload"
                    style="min-width: 110px; margin: 7px 15px 0px 0px; padding: 7px 13px"
                    @click="
                      () =>
                        router.push({ path: '/download/datapack', query: { instance: 'selected' } })
                    "
                    >{{ text('Instance.Resource.Datapack.DownloadNew') }}</PButton
                  >
                  <PButton
                    data-control="BtnManageSelectAll"
                    style="min-width: 110px; margin: 7px 15px 0px 0px; padding: 7px 13px"
                    @click="() => visibleDatapacks.forEach((pack) => selected.add(pack.id))"
                    >{{ text('Instance.Resource.SelectAll') }}</PButton
                  >
                  <PButton
                    data-control="BtnManageInfoExport"
                    style="min-width: 110px; margin: 7px 15px 0px 0px; padding: 7px 13px"
                    :title="text('Instance.Resource.Datapack.ExportInfo.ToolTip')"
                    @click="showUnavailable"
                    >{{ text('Instance.Resource.ExportInfo') }}</PButton
                  >
                </div>
              </PCard>
              <PCard
                data-control="PanListBack"
                style="min-height: 55px; margin: 0px 0px 14px 0px"
                :title="text(' ')"
              >
                <div
                  data-control="PanFilter"
                  style="
                    height: 28px;
                    margin: 0px 0px 0px 15px;
                    display: flex;
                    flex-direction: row;
                    align-items: center;
                  "
                >
                  <label data-control="BtnFilterAll" style="margin: 0px 2px" class="pcl-radio"
                    ><input type="radio" v-model="values.filter" value="All" />{{
                      text('Instance.Resource.Filter.All')
                    }}</label
                  >
                  <label data-control="BtnFilterEnabled" style="margin: 0px 2px" class="pcl-radio"
                    ><input type="radio" v-model="values.filter" value="Enabled" />{{
                      text('Instance.Resource.Filter.Enabled')
                    }}</label
                  >
                  <label data-control="BtnFilterDisabled" style="margin: 0px 2px" class="pcl-radio"
                    ><input type="radio" v-model="values.filter" value="Disabled" />{{
                      text('Instance.Resource.Filter.Disabled')
                    }}</label
                  >
                  <label data-control="BtnFilterCanUpdate" style="margin: 0px 2px" class="pcl-radio"
                    ><input type="radio" v-model="values.filter" value="CanUpdate" />{{
                      text('Instance.Resource.Filter.Updatable')
                    }}</label
                  >
                  <label data-control="BtnFilterError" style="margin: 0px 2px" class="pcl-radio"
                    ><input type="radio" v-model="values.filter" value="Error" />{{
                      text('Instance.Resource.Filter.Error')
                    }}</label
                  >
                </div>
                <div
                  style="
                    height: 28px;
                    margin: 0px 15px 0px 0px;
                    display: flex;
                    flex-direction: row;
                    align-items: center;
                  "
                >
                  <PButton data-control="BtnSort" @click="showUnavailable" inline
                    ><PSvgIcon name="lucide/arrow-up-down" :size="18" />{{
                      text('Instance.Resource.Sort.Text')
                    }}</PButton
                  >
                </div>
                <div
                  data-control="PanList"
                  style="margin: 8px 18px 22px 20px; display: flex; flex-direction: column"
                >
                  <PListItem
                    v-for="pack in visibleDatapacks"
                    :key="pack.id"
                    :title="pack.name"
                    :subtitle="pack.description"
                    ><template #actions
                      ><button
                        class="pcl-icon-button"
                        type="button"
                        :title="
                          text(
                            pack.enabled
                              ? 'Instance.Resource.Filter.Disabled'
                              : 'Instance.Resource.Filter.Enabled',
                          )
                        "
                        :aria-label="
                          text(
                            pack.enabled
                              ? 'Instance.Resource.Filter.Disabled'
                              : 'Instance.Resource.Filter.Enabled',
                          )
                        "
                        @click="toggleItem(pack.id)"
                      >
                        <PSvgIcon
                          :name="pack.enabled ? 'lucide/x' : 'lucide/check'"
                          :size="15"
                        /></button
                      ><PCheckBox
                        :id="`datapack-${pack.id}`"
                        :label="pclText('Proto.Label.Select', [pack.name])"
                        :model-value="selected.has(pack.id)"
                        @update:model-value="(checked) => select(pack.id, checked)" /></template
                  ></PListItem>
                  <p v-if="!visibleDatapacks.length" class="pcl-empty">
                    {{ pclText('Download.Comp.List.NoMatchingResults') }}
                  </p>
                </div>
              </PCard>
            </div>
          </div>
          <PCard
            v-if="!datapacks.length"
            data-control="PanEmpty"
            style="
              margin: 40px;
              grid-row: 1 / span 1;
              grid-column: 1 / span 1;
              justify-self: center;
              align-self: center;
            "
            hide-title
          >
            <div style="margin: 17px 20px; display: flex; flex-direction: column">
              <span
                data-control="TxtEmptyTitle"
                style="margin: 0px 0px 9px 0px; font-size: 19px; color: var(--color-tint)"
                class="pcl-text"
                >{{ text('Instance.Resource.Datapack.Empty.Title') }}</span
              >
              <div style="height: 2px" class="pcl-divider" />
              <span
                data-control="TxtEmptyDescription"
                style="margin: 10px 10px 0px 10px"
                class="pcl-text"
                >{{ text('Instance.Resource.Datapack.Empty.Description') }}</span
              >
              <div
                style="
                  margin: 20px 10px 5px 10px;
                  display: flex;
                  flex-direction: row;
                  align-items: center;
                  flex-wrap: wrap;
                "
              >
                <PButton
                  data-control="BtnHintInstall"
                  style="height: 35px; min-width: 130px; margin: 0px 5px; padding: 0px 12px"
                  @click="resources.importFiles()"
                  type="tint"
                  >{{ text('Instance.Resource.InstallFromFiles') }}</PButton
                >
                <PButton
                  data-control="BtnHintDownload"
                  style="height: 35px; min-width: 130px; margin: 0px 5px; padding: 0px 12px"
                  @click="
                    () =>
                      router.push({ path: '/download/datapack', query: { instance: 'selected' } })
                  "
                  >{{ text('Instance.Resource.Datapack.DownloadNew') }}</PButton
                >
                <PButton
                  data-control="BtnHintOpen"
                  style="height: 35px; min-width: 130px; margin: 0px 5px; padding: 0px 12px"
                  @click="resources.openFolder()"
                  >{{ text('Common.Action.OpenFolder') }}</PButton
                >
              </div>
            </div>
          </PCard>
        </div>
      </div>
    </div>
    <PCard v-if="selected.size" hide-title class="pcl-selection-toolbar"
      ><p>{{ text('Instance.Resource.SelectedCount', [selected.size]) }}</p>
      <div>
        <PButton
          v-for="action in selectionActions"
          :key="action.id"
          inline
          @click="runSelection(action.id)"
          ><PSvgIcon :name="`lucide/${action.icon}`" :size="18" />{{ text(action.key) }}</PButton
        ><PButton inline @click="() => selected.clear()"
          ><PSvgIcon name="lucide/x" :size="18" />{{
            text('Instance.Resource.CancelSelection')
          }}</PButton
        >
      </div></PCard
    >
  </div>
</template>

<style scoped>
.datapack-page {
  display: flex;
  flex-direction: column;
}
</style>
