<script setup lang="ts">
import SetupItem from '@/components/widget/SetupItem.vue'
import { showUnavailable } from '@/util/showUnavailable'
import { pclText } from '@/api/pclLocale'
import { useSelectedInstance } from '@/stores/gameLaunch'
import PListItem from '@/components/widget/PListItem.vue'
import { pluginShowText, instanceIcon } from '@/types/gameInfo'
const instance = useSelectedInstance()
import { moveInstancePageSettings, usePageSettings } from '@/composables/usePageSettings'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PComboBox from '@/components/widget/PComboBox.vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import sideTip from '@/composables/sideTip'
import { useModal } from '@/composables/useModal'
import { useRouter } from 'vue-router'
import { useRepositoriesStore } from '@/stores/repositories'
import { useInstanceFavorites } from '@/composables/useInstanceFavorites'
import { getInstanceIsolation } from '@/api/isolationSettings'
import currentPlatform from '@/util/platform'
import type GameInstance from '@/types/gameInstance'
import { ref, watch } from 'vue'
const { values } = usePageSettings('PageInstanceOverall', {
  ComboDisplayLogo: '0',
  ComboDisplayType: '0',
})
const router = useRouter()
const modal = useModal()
const repos = useRepositoriesStore()
const { isFavorite, toggleFavorite } = useInstanceFavorites()
const busy = ref<string | null>(null)
const description = ref<string | null>(null)
const launchCount = ref<number | null>(null)

watch(
  () => instance.instance_info?.directory,
  async (directory, _, onCleanup) => {
    let current = true
    onCleanup(() => {
      current = false
    })
    description.value = null
    launchCount.value = null
    if (!directory || !isTauri()) return
    try {
      const [savedDescription, count] = await Promise.all([
        invoke<string>('get_instance_description', { instanceDirectory: directory }),
        invoke<number>('get_instance_launch_count', { instanceDirectory: directory }),
      ])
      if (current) {
        description.value = savedDescription || null
        launchCount.value = count
      }
    } catch (error) {
      console.error(error)
    }
  },
  { immediate: true },
)

async function exportScript() {
  const info = instance.instance_info
  if (!info || busy.value || !isTauri()) return
  const windows = currentPlatform === 'windows'
  const target = await save({
    title: pclText('Instance.Overall.Script.SelectSaveTitle'),
    defaultPath: `启动 ${info.name}.${windows ? 'bat' : 'sh'}`,
    filters: windows
      ? [{ name: pclText('Instance.Overall.Script.FileFilter'), extensions: ['bat'] }]
      : [{ name: 'Shell script', extensions: ['sh'] }],
  })
  if (!target) return
  busy.value = 'script'
  sideTip.show(pclText('Instance.Overall.Script.ExportingWarning'))
  try {
    await invoke('export_launch_script', {
      instanceDirectory: info.directory,
      targetPath: target,
    })
    sideTip.show(pclText('Minecraft.Launch.ExportScript.Success'), 'success')
  } catch (error) {
    sideTip.show(`${pclText('Instance.Overall.Script.ExportFailed')}: ${error}`, 'warn')
  } finally {
    busy.value = null
  }
}

async function repairFiles() {
  const info = instance.instance_info
  if (!info || busy.value || !isTauri()) return
  busy.value = 'repair'
  sideTip.show(pclText('Instance.Overall.Repair.Processing'))
  try {
    await invoke('repair_instance_files', { instanceDirectory: info.directory })
    sideTip.show(pclText('Instance.Overall.Repair.Success'), 'success')
  } catch (error) {
    sideTip.show(`${pclText('Instance.Overall.Repair.Failed')} ${error}`, 'warn')
  } finally {
    busy.value = null
  }
}

async function renameInstance() {
  const info = instance.instance_info
  if (!info || busy.value || !isTauri()) return
  const result = await modal.open({
    title: pclText('Instance.Overall.Name.EditTitle'),
    showInput: true,
    defaultInputText: info.name,
    buttons: [
      { content: pclText('Common.Action.Confirm'), operation: () => modal.close(true) },
      { content: pclText('Common.Action.Cancel'), operation: () => modal.close(false) },
    ],
  })
  const newName = (result.input || '').trim()
  if (!result.confirmed || !newName || newName === info.name) return
  busy.value = 'rename'
  try {
    const renamed = await invoke<GameInstance>('rename_instance', {
      instanceDirectory: info.directory,
      newName,
    })
    if (isFavorite(info.directory)) {
      toggleFavorite(info.directory)
      if (!isFavorite(renamed.directory)) toggleFavorite(renamed.directory)
    }
    moveInstancePageSettings(info.directory, renamed.directory)
    delete repos.instancesByPath[info.global_dir]
    instance.setInstance(renamed)
    sideTip.show(pclText('Instance.Overall.Name.RenameSuccess'), 'success')
  } catch (error) {
    sideTip.show(`${pclText('Instance.Overall.Error.OperationFailed')} ${error}`, 'warn')
  } finally {
    busy.value = null
  }
}

async function editDescription() {
  const info = instance.instance_info
  if (!info || busy.value || !isTauri()) return
  const result = await modal.open({
    title: pclText('Instance.Overall.Description.EditTitle'),
    content: pclText('Instance.Overall.Description.EditMessage'),
    showInput: true,
    defaultInputText: description.value ?? undefined,
    buttons: [
      { content: pclText('Common.Action.Confirm'), operation: () => modal.close(true) },
      { content: pclText('Common.Action.Cancel'), operation: () => modal.close(false) },
    ],
  })
  if (!result.confirmed) return
  busy.value = 'description'
  try {
    description.value =
      (await invoke<string>('set_instance_description', {
        instanceDirectory: info.directory,
        description: result.input || '',
      })) || null
  } catch (error) {
    sideTip.show(`${pclText('Instance.Overall.Error.OperationFailed')} ${error}`, 'warn')
  } finally {
    busy.value = null
  }
}

function toggleStar() {
  const info = instance.instance_info
  if (!info) return
  const error = toggleFavorite(info.directory)
  if (error) sideTip.show(error, 'warn')
}

async function deleteInstance() {
  const info = instance.instance_info
  if (!info || busy.value || !isTauri()) return
  if (!repos.repositories) await repos.fetchFromBackend().catch(() => {})
  const repositoryIndex =
    repos.repositories?.findIndex((repo) => repo.path === info.global_dir) ?? -1
  if (repositoryIndex < 0) {
    sideTip.show(pclText('Instance.Overall.Error.OperationFailed'), 'warn')
    return
  }
  let isolated: boolean
  try {
    isolated = await getInstanceIsolation(info.directory)
  } catch (error) {
    sideTip.show(`${pclText('Instance.Overall.Error.OperationFailed')} ${error}`, 'warn')
    return
  }
  const message = isolated
    ? 'Instance.Overall.Delete.ConfirmMessageIsolated'
    : 'Instance.Overall.Delete.ConfirmMessage'
  const result = await modal.open({
    title: pclText('Instance.Overall.Delete.ConfirmTitle'),
    content: pclText(message, [info.name]),
    buttons: [
      {
        content: pclText('Common.Action.Delete'),
        type: 'warn',
        operation: () => modal.close(true),
      },
      { content: pclText('Common.Action.Cancel'), operation: () => modal.close(false) },
    ],
  })
  if (!result.confirmed) return
  busy.value = 'delete'
  try {
    await invoke('instance_folder_action', {
      repository_index: repositoryIndex,
      instance_name: info.name,
      instance_directory: info.directory,
      delete: true,
    })
    if (isFavorite(info.directory)) toggleFavorite(info.directory)
    delete repos.instancesByPath[info.global_dir]
    instance.setInstance(null)
    sideTip.show(pclText('Instance.Overall.Delete.RecycleBinSuccess', [info.name]), 'success')
    await router.push('/instance_select')
  } catch (error) {
    sideTip.show(`${pclText('Instance.Overall.Error.OperationFailed')} ${error}`, 'warn')
  } finally {
    busy.value = null
  }
}
async function openFolder(kind: 'instance' | 'saves' | 'mods') {
  const instanceDirectory = instance.instance_info?.directory
  if (!instanceDirectory || !isTauri()) return
  try {
    await invoke('open_instance_folder', { instanceDirectory, kind })
  } catch (error) {
    sideTip.show(pclText('Proto.Error.OpenFolder', [String(error)]), 'warn')
  }
}
</script>
<template>
  <div class="pcl-page">
    <PCard hide-title>
      <div style="margin: 7px 10px">
        <PListItem
          v-if="instance.instance_info"
          :title="instance.instance_info.name"
          :subtitle="description || instance.instance_info.version"
          :icon="
            instanceIcon(
              instance.instance_info.version,
              instance.instance_info.pluginsVersion || {},
            )
          "
        />
        <p v-else class="pcl-empty">{{ pclText('Proto.Instance.SelectFirst') }}</p>
      </div>
    </PCard>
    <PCard :title="pclText('Instance.Overall.Info.Title')">
      <dl v-if="instance.instance_info" class="instance-info">
        <dt>{{ pclText('Instance.Overall.Info.LaunchCount.Title') }}</dt>
        <dd>
          {{
            launchCount === null
              ? pclText('Instance.Overall.Info.Loading')
              : launchCount === 0
                ? pclText('Instance.Overall.Info.LaunchCount.Never')
                : pclText('Instance.Overall.Info.LaunchCount.Count', [launchCount])
          }}
        </dd>
        <dt>Minecraft</dt>
        <dd>{{ instance.instance_info.version }}</dd>
        <template v-for="(version, loader) in instance.instance_info.pluginsVersion" :key="loader">
          <dt>{{ pluginShowText[loader] || loader }}</dt>
          <dd>{{ version }}</dd>
        </template>
        <dt>{{ pclText('Instance.Overall.Shortcuts.InstanceFolder') }}</dt>
        <dd class="instance-path">{{ instance.instance_info.directory }}</dd>
      </dl>
    </PCard>
    <PCard :title="pclText('Instance.Overall.Personalization.Title')">
      <div class="pcl-card-body">
        <div class="settings-grid">
          <SetupItem :label="pclText('Instance.Overall.Icon.Title')" v-slot="{ id }">
            <PComboBox
              :input-id="id"
              :options="[
                { key: '0', text: pclText('Common.Option.Auto') },
                { key: '1', text: pclText('Common.Option.Customize') },
                { key: '2', text: pclText('Instance.Overall.Icon.Cobblestone') },
                { key: '3', text: pclText('Instance.Overall.Icon.CommandBlock') },
                { key: '4', text: pclText('Instance.Overall.Icon.GoldBlock') },
                { key: '5', text: pclText('Instance.Overall.Icon.GrassBlock') },
                { key: '6', text: pclText('Instance.Overall.Icon.DirtPath') },
                { key: '7', text: pclText('Instance.Overall.Icon.Anvil') },
                { key: '8', text: pclText('Instance.Overall.Icon.RedstoneBlock') },
                { key: '9', text: pclText('Instance.Overall.Icon.RedstoneLampOn') },
                { key: '10', text: pclText('Instance.Overall.Icon.RedstoneLampOff') },
                { key: '11', text: pclText('Instance.Overall.Icon.Egg') },
                { key: '12', text: pclText('Instance.Overall.Icon.Fabric') },
                { key: '13', text: pclText('Instance.Overall.Icon.Quilt') },
                { key: '14', text: pclText('Instance.Overall.Icon.NeoForgeFox') },
                { key: '15', text: pclText('Instance.Overall.Icon.Cleanroom') },
              ]"
              v-model="values.ComboDisplayLogo"
            />
          </SetupItem>
          <SetupItem :label="pclText('Instance.Overall.Category.Title')" v-slot="{ id }">
            <PComboBox
              :input-id="id"
              :options="[
                { key: '0', text: pclText('Common.Option.Auto') },
                {
                  key: '1',
                  text: pclText('Instance.Overall.Category.Hidden'),
                  title: pclText('Instance.Overall.Category.Hidden.ToolTip'),
                },
                { key: '2', text: pclText('Instance.Overall.Category.Modable') },
                { key: '3', text: pclText('Instance.Overall.Category.Regular') },
                { key: '4', text: pclText('Instance.Overall.Category.LessUsed') },
                { key: '5', text: pclText('Instance.Overall.Category.AprilFools') },
              ]"
              v-model="values.ComboDisplayType"
            />
          </SetupItem>
        </div>
        <div class="pcl-button-row pcl-button-row-fill pcl-button-row-spaced">
          <PButton
            style="min-width: 140px; padding: 0px 13px"
            :disabled="!instance.instance_info || !!busy"
            @click="renameInstance"
            >{{ pclText('Instance.Overall.Rename') }}</PButton
          >
          <PButton
            style="min-width: 140px; padding: 0px 13px"
            :disabled="!instance.instance_info || !!busy"
            @click="editDescription"
            >{{ pclText('Instance.Overall.EditDescription') }}</PButton
          >
          <PButton
            style="min-width: 140px; padding: 0px 13px"
            :disabled="!instance.instance_info"
            @click="toggleStar"
            >{{
              instance.instance_info && isFavorite(instance.instance_info.directory)
                ? pclText('Instance.Overall.Unfavorite')
                : pclText('Instance.Overall.Favorite')
            }}</PButton
          >
        </div>
      </div>
    </PCard>
    <PCard :title="pclText('Instance.Overall.Shortcuts.Title')">
      <div class="pcl-card-body">
        <div class="pcl-button-row pcl-button-row-fill">
          <PButton
            :disabled="!instance.instance_info || !isTauri()"
            style="min-width: 140px; padding: 0px 13px"
            @click="openFolder('instance')"
            >{{ pclText('Instance.Overall.Shortcuts.InstanceFolder') }}</PButton
          >
          <PButton
            :disabled="!instance.instance_info || !isTauri()"
            style="min-width: 140px; padding: 0px 13px"
            @click="openFolder('saves')"
            >{{ pclText('Instance.Overall.Shortcuts.SavesFolder') }}</PButton
          >
          <PButton
            :disabled="!instance.instance_info || !isTauri()"
            style="min-width: 140px; padding: 0px 13px"
            @click="openFolder('mods')"
            >{{ pclText('Instance.Overall.Shortcuts.ModsFolder') }}</PButton
          >
        </div>
      </div>
    </PCard>
    <PCard :title="pclText('Instance.Overall.Manage.Title')">
      <div class="pcl-card-body">
        <div class="pcl-button-row">
          <PButton
            style="height: 35px; min-width: 140px; padding: 0px 13px"
            :disabled="!instance.instance_info || !!busy"
            @click="exportScript"
            >{{ pclText('Instance.Overall.Manage.ExportLaunchScript') }}</PButton
          >
          <PButton
            style="height: 35px; min-width: 140px; padding: 0px 13px"
            @click="showUnavailable"
            >{{ pclText('Instance.Overall.Manage.TestGame') }}</PButton
          >
          <PButton
            style="height: 35px; min-width: 140px; padding: 0px 13px"
            :title="pclText('Instance.Overall.Manage.RepairFiles.ToolTip')"
            :disabled="!instance.instance_info || !!busy"
            @click="repairFiles"
            >{{ pclText('Instance.Overall.Manage.RepairFiles') }}</PButton
          >
          <PButton
            style="height: 35px; min-width: 140px; padding: 0px 13px"
            :title="pclText('Instance.Overall.Manage.Reinstall.ToolTip')"
            @click="showUnavailable"
            >{{ pclText('Common.Action.Reset') }}</PButton
          >
          <PButton
            style="height: 35px; min-width: 140px; padding: 0px 13px"
            :disabled="!instance.instance_info || !!busy"
            @click="deleteInstance"
            type="warn"
            >{{ pclText('Instance.Overall.Manage.Delete') }}</PButton
          >
          <PButton
            style="height: 35px; min-width: 140px; padding: 0px 13px"
            @click="showUnavailable"
            >{{ pclText('Instance.Overall.Manage.PatchCore') }}</PButton
          >
        </div>
      </div>
    </PCard>
  </div>
</template>

<style scoped>
.instance-info {
  display: grid;
  grid-template-columns: max-content minmax(0, 1fr);
  gap: 8px 24px;
  margin: 0 25px 15px;
}
.instance-info dt {
  color: var(--color-text-grey);
}
.instance-info dd {
  margin: 0;
}
.instance-path {
  overflow-wrap: anywhere;
}
</style>
