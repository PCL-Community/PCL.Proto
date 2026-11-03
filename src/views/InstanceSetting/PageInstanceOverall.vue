<script setup lang="ts">
import { showUnavailable } from '@/util/showUnavailable'
import { pclText } from '@/api/pclLocale'
import { useSelectedInstance } from '@/stores/gameLaunch'
import PListItem from '@/components/widget/PListItem.vue'
import { pluginShowText, instanceIcon } from '@/types/gameInfo'
const instance = useSelectedInstance()
import { usePageSettings } from '@/composables/usePageSettings'
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
const description = ref('')

const text = pclText

watch(
  () => instance.instance_info?.directory,
  async (directory) => {
    description.value = ''
    if (!directory || !isTauri()) return
    try {
      description.value = await invoke<string>('get_instance_description', {
        instanceDirectory: directory,
      })
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
    title: text('Instance.Overall.Script.SelectSaveTitle'),
    defaultPath: `启动 ${info.name}.${windows ? 'bat' : 'sh'}`,
    filters: windows
      ? [{ name: text('Instance.Overall.Script.FileFilter'), extensions: ['bat'] }]
      : [{ name: 'Shell script', extensions: ['sh'] }],
  })
  if (!target) return
  busy.value = 'script'
  sideTip.show(text('Instance.Overall.Script.ExportingWarning'))
  try {
    await invoke('export_launch_script', {
      instanceDirectory: info.directory,
      targetPath: target,
    })
    sideTip.show(text('Minecraft.Launch.ExportScript.Success'), 'success')
  } catch (error) {
    sideTip.show(`${text('Instance.Overall.Script.ExportFailed')}: ${error}`, 'warn')
  } finally {
    busy.value = null
  }
}

async function repairFiles() {
  const info = instance.instance_info
  if (!info || busy.value || !isTauri()) return
  busy.value = 'repair'
  sideTip.show(text('Instance.Overall.Repair.Processing'))
  try {
    await invoke('repair_instance_files', { instanceDirectory: info.directory })
    sideTip.show(text('Instance.Overall.Repair.Success'), 'success')
  } catch (error) {
    sideTip.show(`${text('Instance.Overall.Repair.Failed')} ${error}`, 'warn')
  } finally {
    busy.value = null
  }
}

async function renameInstance() {
  const info = instance.instance_info
  if (!info || busy.value || !isTauri()) return
  const result = await modal.open({
    title: text('Instance.Overall.Name.EditTitle'),
    showInput: true,
    defaultInputText: info.name,
    buttons: [
      { content: text('Common.Action.Confirm'), operation: () => modal.close(true) },
      { content: text('Common.Action.Cancel'), operation: () => modal.close(false) },
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
    instance.setInstance(renamed)
    sideTip.show(text('Instance.Overall.Name.RenameSuccess'), 'success')
  } catch (error) {
    sideTip.show(`${text('Instance.Overall.Error.OperationFailed')} ${error}`, 'warn')
  } finally {
    busy.value = null
  }
}

async function editDescription() {
  const info = instance.instance_info
  if (!info || busy.value || !isTauri()) return
  const result = await modal.open({
    title: text('Instance.Overall.Description.EditTitle'),
    content: text('Instance.Overall.Description.EditMessage'),
    showInput: true,
    defaultInputText: description.value,
    buttons: [
      { content: text('Common.Action.Confirm'), operation: () => modal.close(true) },
      { content: text('Common.Action.Cancel'), operation: () => modal.close(false) },
    ],
  })
  if (!result.confirmed) return
  busy.value = 'description'
  try {
    description.value = await invoke<string>('set_instance_description', {
      instanceDirectory: info.directory,
      description: result.input || '',
    })
  } catch (error) {
    sideTip.show(`${text('Instance.Overall.Error.OperationFailed')} ${error}`, 'warn')
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
    sideTip.show(text('Instance.Overall.Error.OperationFailed'), 'warn')
    return
  }
  const isolated = await getInstanceIsolation(info.directory).catch(() => false)
  const message = isolated
    ? 'Instance.Overall.Delete.ConfirmMessageIsolated'
    : 'Instance.Overall.Delete.ConfirmMessage'
  const result = await modal.open({
    title: text('Instance.Overall.Delete.ConfirmTitle'),
    content: text(message, [info.name]),
    buttons: [
      { content: text('Common.Action.Delete'), type: 'warn', operation: () => modal.close(true) },
      { content: text('Common.Action.Cancel'), operation: () => modal.close(false) },
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
    instance.setInstance(null)
    sideTip.show(text('Instance.Overall.Delete.RecycleBinSuccess', [info.name]), 'success')
    await router.push('/instance_select')
  } catch (error) {
    sideTip.show(`${text('Instance.Overall.Error.OperationFailed')} ${error}`, 'warn')
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
          :subtitle="instance.instance_info.version"
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
    <PCard :title="text('Instance.Overall.Info.Title')">
      <div style="margin: 0px 25px 5px 15px; display: flex; flex-direction: column">
        <p v-if="instance.instance_info">Minecraft {{ instance.instance_info.version }}</p>
        <p v-for="(version, loader) in instance.instance_info?.pluginsVersion" :key="loader">
          {{ pluginShowText[loader] || loader }} {{ version }}
        </p>
        <p>{{ instance.instance_info?.directory }}</p>
        <p v-if="description">{{ description }}</p>
      </div>
    </PCard>
    <PCard :title="text('Instance.Overall.Personalization.Title')">
      <div style="margin: 0px 25px 15px 25px; display: flex; flex-direction: column">
        <div
          style="
            display: grid;
            grid-template-columns: auto minmax(0, 1fr);
            grid-template-rows: 28px 9px 28px;
          "
        >
          <span
            style="
              margin: 0px 25px 0px 0px;
              grid-row: 1 / span 1;
              grid-column: 1 / span 1;
              justify-self: start;
              align-self: center;
            "
            class="pcl-text"
            >{{ text('Instance.Overall.Icon.Title') }}</span
          >
          <PComboBox
            style="grid-row: 1 / span 1; grid-column: 2 / span 1"
            :options="[
              { key: '0', text: text('Common.Option.Auto') },
              { key: '1', text: text('Common.Option.Customize') },
              { key: '2', text: text('Instance.Overall.Icon.Cobblestone') },
              { key: '3', text: text('Instance.Overall.Icon.CommandBlock') },
              { key: '4', text: text('Instance.Overall.Icon.GoldBlock') },
              { key: '5', text: text('Instance.Overall.Icon.GrassBlock') },
              { key: '6', text: text('Instance.Overall.Icon.DirtPath') },
              { key: '7', text: text('Instance.Overall.Icon.Anvil') },
              { key: '8', text: text('Instance.Overall.Icon.RedstoneBlock') },
              { key: '9', text: text('Instance.Overall.Icon.RedstoneLampOn') },
              { key: '10', text: text('Instance.Overall.Icon.RedstoneLampOff') },
              { key: '11', text: text('Instance.Overall.Icon.Egg') },
              { key: '12', text: text('Instance.Overall.Icon.Fabric') },
              { key: '13', text: text('Instance.Overall.Icon.Quilt') },
              { key: '14', text: text('Instance.Overall.Icon.NeoForgeFox') },
              { key: '15', text: text('Instance.Overall.Icon.Cleanroom') },
            ]"
            v-model="values.ComboDisplayLogo"
          />
          <span
            style="
              margin: 0px 25px 0px 0px;
              grid-row: 3 / span 1;
              grid-column: 1 / span 1;
              justify-self: start;
              align-self: center;
            "
            class="pcl-text"
            >{{ text('Instance.Overall.Category.Title') }}</span
          >
          <PComboBox
            style="grid-row: 3 / span 1; grid-column: 2 / span 1"
            :options="[
              { key: '0', text: text('Common.Option.Auto') },
              {
                key: '1',
                text: text('Instance.Overall.Category.Hidden'),
                title: text('Instance.Overall.Category.Hidden.ToolTip'),
              },
              { key: '2', text: text('Instance.Overall.Category.Modable') },
              { key: '3', text: text('Instance.Overall.Category.Regular') },
              { key: '4', text: text('Instance.Overall.Category.LessUsed') },
              { key: '5', text: text('Instance.Overall.Category.AprilFools') },
            ]"
            v-model="values.ComboDisplayType"
          />
        </div>
        <div
          style="
            height: 35px;
            margin: 15px 0px 7px 0px;
            display: grid;
            grid-template-columns: auto auto auto;
          "
        >
          <PButton
            style="
              min-width: 140px;
              margin: 0px 20px 0px 0px;
              padding: 0px 13px;
              grid-row: 1 / span 1;
              grid-column: 1 / span 1;
            "
            :disabled="!instance.instance_info || busy !== ''"
            @click="renameInstance"
            >{{ text('Instance.Overall.Rename') }}</PButton
          >
          <PButton
            style="
              min-width: 140px;
              margin: 0px 20px 0px 0px;
              padding: 0px 13px;
              grid-row: 1 / span 1;
              grid-column: 2 / span 1;
            "
            :disabled="!instance.instance_info || busy !== ''"
            @click="editDescription"
            >{{ text('Instance.Overall.EditDescription') }}</PButton
          >
          <PButton
            style="
              min-width: 140px;
              margin: 0px 20px 0px 0px;
              padding: 0px 13px;
              grid-row: 1 / span 1;
              grid-column: 3 / span 1;
            "
            :disabled="!instance.instance_info"
            @click="toggleStar"
            >{{
              instance.instance_info && isFavorite(instance.instance_info.directory)
                ? text('Instance.Overall.Unfavorite')
                : text('Instance.Overall.Favorite')
            }}</PButton
          >
        </div>
      </div>
    </PCard>
    <PCard :title="text('Instance.Overall.Shortcuts.Title')">
      <div style="margin: 0px 25px 15px 25px; display: flex; flex-direction: column">
        <div
          style="
            height: 35px;
            margin: 2px 0px 7px 0px;
            display: grid;
            grid-template-columns: auto auto auto;
          "
        >
          <PButton
            :disabled="!instance.instance_info || !isTauri()"
            style="
              min-width: 140px;
              margin: 0px 20px 0px 0px;
              padding: 0px 13px;
              grid-row: 1 / span 1;
              grid-column: 1 / span 1;
            "
            @click="openFolder('instance')"
            >{{ text('Instance.Overall.Shortcuts.InstanceFolder') }}</PButton
          >
          <PButton
            :disabled="!instance.instance_info || !isTauri()"
            style="
              min-width: 140px;
              margin: 0px 20px 0px 0px;
              padding: 0px 13px;
              grid-row: 1 / span 1;
              grid-column: 2 / span 1;
            "
            @click="openFolder('saves')"
            >{{ text('Instance.Overall.Shortcuts.SavesFolder') }}</PButton
          >
          <PButton
            :disabled="!instance.instance_info || !isTauri()"
            style="
              min-width: 140px;
              margin: 0px 20px 0px 0px;
              padding: 0px 13px;
              grid-row: 1 / span 1;
              grid-column: 3 / span 1;
            "
            @click="openFolder('mods')"
            >{{ text('Instance.Overall.Shortcuts.ModsFolder') }}</PButton
          >
        </div>
      </div>
    </PCard>
    <PCard :title="text('Instance.Overall.Manage.Title')">
      <div style="margin: 0px 25px 15px 25px; display: flex; flex-direction: column">
        <div
          style="
            margin: -5px -20px 7px 0px;
            display: flex;
            flex-direction: row;
            align-items: center;
            flex-wrap: wrap;
          "
        >
          <PButton
            style="height: 35px; min-width: 140px; margin: 7px 20px 0px 0px; padding: 0px 13px"
            :disabled="!instance.instance_info || busy !== ''"
            @click="exportScript"
            >{{ text('Instance.Overall.Manage.ExportLaunchScript') }}</PButton
          >
          <PButton
            style="height: 35px; min-width: 140px; margin: 7px 20px 0px 0px; padding: 0px 13px"
            @click="showUnavailable"
            >{{ text('Instance.Overall.Manage.TestGame') }}</PButton
          >
          <PButton
            style="height: 35px; min-width: 140px; margin: 7px 20px 0px 0px; padding: 0px 13px"
            :title="text('Instance.Overall.Manage.RepairFiles.ToolTip')"
            :disabled="!instance.instance_info || busy !== ''"
            @click="repairFiles"
            >{{ text('Instance.Overall.Manage.RepairFiles') }}</PButton
          >
          <PButton
            style="height: 35px; min-width: 140px; margin: 7px 20px 0px 0px; padding: 0px 13px"
            :title="text('Instance.Overall.Manage.Reinstall.ToolTip')"
            @click="showUnavailable"
            >{{ text('Common.Action.Reset') }}</PButton
          >
          <PButton
            style="height: 35px; min-width: 140px; margin: 7px 20px 0px 0px; padding: 0px 13px"
            :disabled="!instance.instance_info || busy !== ''"
            @click="deleteInstance"
            type="warn"
            >{{ text('Instance.Overall.Manage.Delete') }}</PButton
          >
          <PButton
            style="height: 35px; min-width: 140px; margin: 7px 20px 0px 0px; padding: 0px 13px"
            @click="showUnavailable"
            >{{ text('Instance.Overall.Manage.PatchCore') }}</PButton
          >
        </div>
      </div>
    </PCard>
  </div>
</template>
