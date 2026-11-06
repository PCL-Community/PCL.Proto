<script setup lang="ts">
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import sideTip from '@/composables/sideTip'
import SideNavLayout from '@/layout/SideNavLayout.vue'
import { useRepositoriesStore } from '@/stores/repositories'
import { computed, h, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { pclText } from '@/api/pclLocale'
import { isTauri } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { importLocalModpack } from '@/api/modpack'
const repo = useRepositoriesStore()
const router = useRouter()
const route = useRoute()
async function add() {
  try {
    await repo.addNew()
  } catch (err) {
    sideTip.show(pclText('Proto.Error.AddFolder', [String(err)]), 'warn')
  }
}
async function importModpack() {
  if (!isTauri()) return sideTip.show(pclText('Proto.Desktop.Download'), 'warn')
  try {
    const path = await open({
      multiple: false,
      filters: [{ name: 'Modrinth', extensions: ['mrpack', 'zip'] }],
    })
    if (!path) return
    await importLocalModpack(path)
  } catch (error) {
    sideTip.show(pclText('Proto.Error.Download', [String(error)]), 'warn')
  }
}
const groups = computed(() => [
  {
    title: pclText('Select.Folder.ListTitle'),
    content:
      repo.repositories?.map((item, index) => ({
        text: item.name,
        subtitle: item.path,
        linkto: `/instance_select/instance_select_sub/${index}`,
      })) || [],
  },
  {
    title: pclText('Select.Folder.AddOrImport'),
    content: [
      {
        text: pclText('Select.Folder.AddExisting.Title'),
        icon: () => h(PSvgIcon, { name: 'lucide/folder-plus' }),
        clickCallback: add,
      },
      {
        text: pclText('Select.Folder.ImportModpack.Title'),
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
