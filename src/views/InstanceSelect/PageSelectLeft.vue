<script setup lang="ts">
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import sideTip from '@/composables/sideTip'
import SideNavLayout from '@/layout/SideNavLayout.vue'
import { useRepositoriesStore } from '@/stores/repositories'
import { computed, h, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { pclText } from '@/api/pclLocale'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useModal } from '@/composables/useModal'
import { isTaskCancelled, useTaskManager } from '@/stores/task'
const repo = useRepositoriesStore()
const router = useRouter()
const route = useRoute()
const text = pclText
async function add() {
  try {
    await repo.addNew()
  } catch (err) {
    sideTip.show(pclText('Proto.Error.AddFolder', [String(err)]), 'warn')
  }
}
async function importModpack() {
  if (!isTauri()) return sideTip.show(text('Proto.Desktop.Download'), 'warn')
  try {
    const path = await open({
      multiple: false,
      filters: [{ name: 'Modrinth', extensions: ['mrpack', 'zip'] }],
    })
    if (!path) return
    const info = await invoke<{
      name: string
      minecraft: string
      fabric: boolean
      optional_files: string[]
    }>('inspect_modpack', { path })
    const choice = await useModal().open({
      title: text('Minecraft.Download.Modpack.InputInstanceName'),
      showInput: true,
      defaultInputText: info.name,
    })
    if (!choice.confirmed || !choice.input?.trim()) return
    const optional: string[] = []
    for (const file of info.optional_files) {
      const result = await useModal().open({
        title: text('Minecraft.Download.Modpack.OptionalFile.Title'),
        content: text('Minecraft.Download.Modpack.OptionalFile.Message', [file]),
        plainText: true,
      })
      if (result.confirmed) optional.push(file)
    }
    const name = choice.input.trim()
    const task = useTaskManager().installModpack(path, name, optional, info.fabric)
    sideTip.show(text('Download.Comp.QuickDownload.Hint.DownloadStarted', [name]))
    await task
    sideTip.show(text('Minecraft.Download.Stage.InstallGame') + ': ' + name, 'success')
  } catch (error) {
    if (!isTaskCancelled(error)) sideTip.show(text('Proto.Error.Download', [String(error)]), 'warn')
  }
}
const groups = computed(() => [
  {
    title: text('Select.Folder.ListTitle'),
    content:
      repo.repositories?.map((item, index) => ({
        text: item.name,
        subtitle: item.path,
        linkto: `/instance_select/instance_select_sub/${index}`,
      })) || [],
  },
  {
    title: text('Select.Folder.AddOrImport'),
    content: [
      {
        text: text('Select.Folder.AddExisting.Title'),
        icon: () => h(PSvgIcon, { name: 'lucide/folder-plus' }),
        clickCallback: add,
      },
      {
        text: text('Select.Folder.ImportModpack.Title'),
        icon: () => h(PSvgIcon, { name: 'lucide/package' }),
        clickCallback: importModpack,
      },
    ],
  },
])
onMounted(async () => {
  try {
    if (!isTauri()) return
    if (!repo.repositories) await repo.fetchFromBackend()
    if (repo.repositories?.length && !route.params.repository)
      await router.replace('/instance_select/instance_select_sub/0')
  } catch (err) {
    sideTip.show(pclText('Proto.Error.Folders', [String(err)]), 'warn')
  }
})
</script>
<template><SideNavLayout :side-nav-groups="groups" /></template>
