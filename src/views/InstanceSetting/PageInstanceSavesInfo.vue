<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { isTauri } from '@tauri-apps/api/core'
import {
  getSaveInfo,
  updateSaveInfo,
  type SaveChanges,
  type SaveInfo,
} from '@/api/instanceResources'
import { pclFormatLocale, pclText } from '@/api/pclLocale'
import { useSelectedInstance } from '@/stores/gameLaunch'
import { openExternal } from '@/util/externalLink'
import sideTip from '@/composables/sideTip'
import PCard from '@/components/widget/PCard.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import PCheckBox from '@/components/widget/PCheckBox.vue'
import PHint from '@/components/widget/PHint.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'

const route = useRoute()
const instance = useSelectedInstance()
const save = computed(() => (typeof route.query.id === 'string' ? route.query.id : ''))
const directory = computed(() => instance.instance_info?.directory)
const info = ref<SaveInfo | null>(null)
const error = ref('')
const loading = ref(false)
const saving = ref(false)
let generation = 0
const text = pclText

watch(
  [directory, save],
  async ([path, name]) => {
    const current = ++generation
    info.value = null
    error.value = ''
    loading.value = false
    if (!isTauri() || !path || !name) return
    loading.value = true
    try {
      const value = await getSaveInfo(path, name)
      if (current === generation) info.value = value
    } catch (failure) {
      if (current === generation)
        error.value = `${text('Instance.Saves.Info.Error.LoadFailed')}: ${failure}`
    } finally {
      if (current === generation) loading.value = false
    }
  },
  { immediate: true },
)
onMounted(() => {
  if (!instance.instance_info && isTauri())
    void instance.fetch().catch((e) => {
      error.value = String(e)
    })
})
onUnmounted(() => generation++)

async function update(changes: SaveChanges) {
  const path = directory.value
  const name = save.value
  if (!path || !name || saving.value) return
  const current = generation
  saving.value = true
  try {
    const value = await updateSaveInfo(path, name, changes)
    if (current === generation) info.value = value
    if (current === generation)
      sideTip.show(
        text(
          changes.allowCommands === undefined
            ? 'Instance.Saves.Info.Modify.DifficultySuccess'
            : 'Instance.Saves.Info.Modify.CheatSuccess',
        ),
        'success',
      )
  } catch (failure) {
    if (current === generation)
      sideTip.show(
        `${text(
          changes.allowCommands === undefined
            ? 'Instance.Saves.Info.Modify.DifficultyFailed'
            : 'Instance.Saves.Info.Modify.CheatFailed',
        )}: ${failure}`,
        'warn',
      )
  } finally {
    saving.value = false
  }
}

const modes = ['Survival', 'Creative', 'Adventure', 'Spectator', 'Hardcore']
const difficulties = ['Peaceful', 'Easy', 'Normal', 'Hard']
const difficultyOptions = computed(() =>
  difficulties.map((value, index) => ({
    key: String(index),
    text: text(`Instance.Saves.Info.Difficulty.${value}`),
  })),
)
const commandOptions = computed(() => [
  { key: 'false', text: text('Instance.Saves.Info.AllowCommands.NotAllowed') },
  { key: 'true', text: text('Instance.Saves.Info.AllowCommands.Allowed') },
])
const details = computed<[string, string][]>(() => {
  const data = info.value
  if (!data) return []
  const rows: [string, string][] = [
    ...(data.versionName
      ? [
          [
            text('Instance.Saves.Info.Version'),
            `${data.versionName}${data.versionId === null ? '' : ` (${data.versionId})`}`,
          ] as [string, string],
        ]
      : []),
    [text('Instance.Saves.Info.LevelName'), data.levelName],
    [
      text('Instance.Saves.Info.LastPlayed'),
      data.lastPlayed
        ? new Date(data.lastPlayed).toLocaleString(pclFormatLocale.value)
        : text('Instance.Saves.Info.GetFailed'),
    ],
    ...(data.spawn
      ? [[text('Instance.Saves.Info.SpawnPoint'), data.spawn.join(' / ')] as [string, string]]
      : []),
    [
      text('Instance.Saves.Info.GameMode'),
      text(`Instance.Saves.Info.GameMode.${modes[data.gameMode] || 'Survival'}`),
    ],
    [
      text('Instance.Saves.Info.PlayTime'),
      `${Math.floor(data.playTicks / 20 / 3600)}h ${Math.floor(data.playTicks / 20 / 60) % 60}m`,
    ],
  ]
  return rows
})

async function copySeed() {
  if (!info.value?.seed) return
  try {
    await navigator.clipboard.writeText(info.value.seed)
  } catch (failure) {
    sideTip.show(`${text('Instance.Saves.Info.Error.ClipboardFailed')}: ${failure}`, 'warn')
  }
}
function openSeedMap() {
  const { seed, versionName } = info.value || {}
  if (!versionName) {
    sideTip.show(text('Instance.Saves.Info.Chunkbase.UnknownVersion'), 'warn')
    return
  }
  if (/[a-z]/i.test(versionName)) {
    sideTip.show(text('Instance.Saves.Info.Chunkbase.PreviewVersion', [versionName]), 'warn')
    return
  }
  const version = (
    versionName.startsWith('1.21') ? versionName : versionName.split('.').slice(0, 2).join('.')
  ).replaceAll('.', '_')
  void openExternal(
    `https://www.chunkbase.com/apps/seed-map#seed=${encodeURIComponent(seed || '')}&platform=java_${encodeURIComponent(version)}&dimension=overworld`,
  )
}
</script>

<template>
  <div class="pcl-page">
    <PLoading v-if="loading" :loading-text="text('Instance.Saves.Status.Refreshing')" />
    <PHint v-else-if="error" severity="error">{{ error }}</PHint>
    <template v-else-if="info">
      <PHint v-if="!info.versionName" severity="info">{{
        text(
          info.difficulty !== null
            ? 'Instance.Saves.Info.VersionHint.1_9'
            : info.canEditCommands
              ? 'Instance.Saves.Info.VersionHint.1_8'
              : 'Instance.Saves.Info.VersionHint.1_3',
        )
      }}</PHint>
      <PCard :title="text('Instance.Saves.Info.Details.Title')">
        <dl class="details">
          <template v-for="[label, value] in details" :key="label"
            ><dt>{{ label }}</dt>
            <dd>{{ value }}</dd></template
          >
          <dt>{{ text('Instance.Saves.Info.Seed') }}</dt>
          <dd class="seed">
            <button v-if="info.seed !== null" class="seed-value" type="button" @click="copySeed">
              {{ info.seed }}
            </button>
            <span v-else>{{ text('Instance.Saves.Info.GetFailed') }}</span>
            <button
              v-if="info.seed !== null"
              class="pcl-icon-button"
              type="button"
              :title="text('Instance.Saves.Info.Chunkbase.ToolTip')"
              :aria-label="text('Instance.Saves.Info.Chunkbase.ToolTip')"
              @click="openSeedMap"
            >
              <PSvgIcon name="lucide/external-link" :size="15" />
            </button>
          </dd>
        </dl>
      </PCard>
      <PCard
        v-if="info.canEditCommands || info.canEditDifficulty"
        :title="text('Instance.Saves.Info.Settings.Title')"
      >
        <div class="settings" :title="text('Instance.Saves.Info.Modify.BeforeSave')">
          <template v-if="info.canEditCommands">
            <span>{{ text('Instance.Saves.Info.AllowCommands') }}</span>
            <PComboBox
              :model-value="String(info.allowCommands)"
              :options="commandOptions"
              :disabled="saving"
              :aria-label="text('Instance.Saves.Info.AllowCommands')"
              @update:model-value="(value) => update({ allowCommands: value === 'true' })"
            />
          </template>
          <template v-if="info.canEditDifficulty && info.difficulty !== null">
            <span>{{ text('Instance.Saves.Info.GameDifficultyLabel') }}</span>
            <div class="difficulty">
              <PComboBox
                :model-value="String(info.difficulty)"
                :options="difficultyOptions"
                :disabled="saving"
                :aria-label="text('Instance.Saves.Info.GameDifficultyLabel')"
                @update:model-value="(value) => update({ difficulty: Number(value) })"
              />
              <PCheckBox
                v-if="info.canEditLock && !info.hardcore"
                :model-value="info.difficultyLocked"
                :disabled="saving"
                :label="text('Instance.Saves.Info.LockDifficulty')"
                :title="text('Instance.Saves.Info.LockDifficulty.ToolTip')"
                @update:model-value="(value) => update({ difficultyLocked: value })"
              />
            </div>
          </template>
        </div>
      </PCard>
    </template>
    <PCard v-else class="pcl-empty-state-card" hide-title>
      <div class="pcl-empty-state">
        <h2 class="pcl-empty-state-title">{{ text('Proto.Empty.SaveInfo') }}</h2>
        <div class="pcl-divider" />
      </div>
    </PCard>
  </div>
</template>

<style scoped>
.details,
.settings {
  margin: 0 15px 15px;
  display: grid;
  grid-template-columns: max-content minmax(0, 1fr);
  gap: 6px 24px;
  align-items: center;
}
dd {
  margin: 0;
  overflow-wrap: anywhere;
}
.seed,
.difficulty {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.seed-value {
  border: 0;
  padding: 0;
  color: var(--color-tint);
  background: transparent;
  cursor: pointer;
  font: inherit;
}
.settings :deep(.dropdown-wrapper) {
  width: 120px;
}
</style>
