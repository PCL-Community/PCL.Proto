<script setup lang="ts">
import { showUnavailable } from '@/util/showUnavailable'
import { computed, ref } from 'vue'
import PCard from '@/components/widget/PCard.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PSearchBox from '@/components/widget/PSearchBox.vue'
import PSelectionToolbar from '@/components/widget/PSelectionToolbar.vue'
import PCompItem from '@/components/widget/PCompItem.vue'
import PButton from '@/components/widget/PButton.vue'
import PIconTextButton from '@/components/widget/PIconTextButton.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import PCheckBox from '@/components/widget/PCheckBox.vue'
import sideTip from '@/composables/sideTip'
import { useCompFavorites } from '@/stores/compFavorites'
import { pclText } from '@/api/pclLocale'
const favorites = useCompFavorites()
const query = ref('')
const folder = ref('default')

const selected = ref(new Set<string>())
const selectedItems = computed(() =>
  favorites.items.filter((item) => selected.value.has(item.project_id)),
)
function select(id: string, checked: boolean) {
  if (checked) selected.value.add(id)
  else selected.value.delete(id)
}
async function share(ids: string[]) {
  if (!ids.length) {
    sideTip.show(pclText('Download.Comp.Favorites.Hint.NothingShared'))
    return
  }
  try {
    await navigator.clipboard.writeText(JSON.stringify(ids))
    sideTip.show(pclText('Common.Hint.Copied'), 'success')
  } catch {
    sideTip.show(pclText('Proto.Clipboard.CopyFailed'), 'warn')
  }
}
function unfavoriteSelected() {
  selectedItems.value.forEach((item) => favorites.toggle(item))
  selected.value.clear()
}
const items = computed(() =>
  favorites.items.filter((item) =>
    `${item.title} ${item.description}`.toLowerCase().includes(query.value.toLowerCase()),
  ),
)
</script>
<template>
  <section class="pcl-page favorites-page">
    <div class="search-row">
      <PSearchBox
        v-model="query"
        :placeholder="pclText('Download.Comp.Favorites.Search.Hint')"
      /><PCard hide-title class="folder-card" overflow-visible
        ><div class="folder-row">
          <PComboBox
            v-model="folder"
            :options="[
              { key: 'default', text: pclText('Download.Comp.Detail.Favorites.DefaultName') },
            ]"
          />
          <details class="manage-menu">
            <summary
              :aria-label="pclText('Download.Comp.Favorites.Manage')"
              :title="pclText('Download.Comp.Favorites.Manage')"
            >
              <PSvgIcon name="lucide/settings" :size="18" />
            </summary>
            <div class="menu-actions">
              <PButton
                v-for="action in ['New', 'Rename', 'Delete', 'Import']"
                :key="action"
                inline
                @click="showUnavailable"
                >{{ pclText(`Download.Comp.Favorites.Menu.${action}`) }}</PButton
              >
              <PButton
                inline
                @click="() => share(favorites.items.map((item) => item.project_id))"
                >{{ pclText('Download.Comp.Favorites.Menu.Share') }}</PButton
              >
            </div>
          </details>
        </div></PCard
      >
    </div>
    <PCard
      v-if="items.length"
      :title="
        pclText('Download.Comp.Favorites.CardTitle', [
          pclText('Download.Comp.Detail.Favorites.DefaultName'),
          items.length,
        ])
      "
      ><div class="favorite-items">
        <div v-for="item in items" :key="item.project_id" class="favorite-row">
          <PCheckBox
            :id="`favorite-${item.project_id}`"
            :model-value="selected.has(item.project_id)"
            :label="pclText('Proto.Label.Select', [item.title])"
            class="select-favorite"
            @update:model-value="(checked) => select(item.project_id, checked)"
          />
          <PCompItem :data="item" clickable />
        </div></div></PCard
    ><PCard v-else hide-title class="empty-card"
      ><div class="empty-content">
        <h2>{{ pclText('Download.Comp.Favorites.Empty.Title') }}</h2>
        <hr />
        <p>
          {{
            query
              ? pclText('Proto.Empty.Resources')
              : pclText('Download.Comp.Favorites.Empty.Description')
          }}
        </p>
      </div></PCard
    >
    <PSelectionToolbar
      :count="selectedItems.length"
      :label="pclText('Download.Comp.Favorites.Hint.SelectedCount', [selectedItems.length])"
    >
      <PIconTextButton icon="lucide/heart-filled" @click="unfavoriteSelected">{{
        pclText('Download.Comp.Favorites.Cancel')
      }}</PIconTextButton>
      <PIconTextButton
        icon="lucide/share-2"
        @click="() => share(selectedItems.map((item) => item.project_id))"
        >{{ pclText('Download.Comp.Favorites.Share') }}</PIconTextButton
      >
      <PIconTextButton icon="lucide/download" @click="showUnavailable">{{
        pclText('Download.Comp.Favorites.Download')
      }}</PIconTextButton>
      <PIconTextButton icon="lucide/x" @click="() => selected.clear()">{{
        pclText('Download.Comp.Favorites.Deselect')
      }}</PIconTextButton>
    </PSelectionToolbar>
  </section>
</template>
<style scoped>
.search-row {
  display: flex;
  gap: 15px;
  margin-bottom: 15px;
}
.search-row > :first-child {
  flex: 1;
}
.folder-row {
  padding: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
}
.manage-menu summary {
  cursor: pointer;
  list-style: none;
}
.manage-menu {
  position: relative;
}
.menu-actions {
  position: absolute;
  right: 0;
  top: 100%;
  z-index: 2;
  min-width: 160px;
  background: var(--color-background);
  box-shadow: var(--box-shadow);
  border-radius: 5px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 8px;
}
.select-favorite :deep(label) {
  font-size: 0;
}
.favorite-items {
  padding: 0 20px 15px;
}
.favorite-row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.favorite-row > .comp-item {
  flex: 1;
  min-width: 0;
}
.empty-card {
  margin: 40px 15px;
}
.empty-content {
  padding: 17px 20px;
}
.empty-content h2 {
  font-size: 19px;
  font-weight: normal;
  color: var(--color-tint);
}
.empty-content hr {
  margin: 9px 0 15px;
  border: 0;
  height: 2px;
  background: var(--color-tint-light);
}
.empty-content p {
  padding: 0 10px;
}
</style>
