<script setup lang="ts">
import SideNavLayout from '@/layout/SideNavLayout.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import { computed, h } from 'vue'
import { useRoute } from 'vue-router'
import { pclText } from '@/api/pclLocale'
import sideTip from '@/composables/sideTip'
import { listInstanceResources, openInstancePath } from '@/api/instanceResources'
import { useSelectedInstance } from '@/stores/gameLaunch'
import { isTauri } from '@tauri-apps/api/core'
const text = pclText
const route = useRoute()
const instance = useSelectedInstance()
const save = computed(() => (typeof route.query.id === 'string' ? route.query.id : ''))
const link = (path: string) => `${path}?id=${encodeURIComponent(save.value)}`
async function openFolder() {
  const directory = instance.instance_info?.directory
  if (!isTauri() || !directory || !save.value) return
  try {
    const path = await listInstanceResources(directory, 'save')
    const entry = path.find((item) => item.name === save.value)
    if (!entry) throw new Error(text('Instance.Saves.Info.Error.LoadFailed'))
    await openInstancePath(directory, entry.path)
  } catch (error) {
    sideTip.show(`${text('Instance.Saves.Error.OperationFailed')} ${error}`, 'warn')
  }
}
const groups = computed(() => [
  {
    content: [
      {
        text: text('Instance.Saves.Left.Info'),
        linkto: link('/instance_save/info'),
        icon: () => h(PSvgIcon, { name: 'lucide/info' }),
      },
      {
        text: text('Instance.Saves.Left.Datapack'),
        linkto: link('/instance_save/datapack'),
        icon: () => h(PSvgIcon, { name: 'lucide/file-archive' }),
      },
    ],
  },
  {
    title: text('Instance.Saves.Left.QuickActions'),
    content: [
      {
        text: text('Common.Action.OpenFolder'),
        icon: () => h(PSvgIcon, { name: 'lucide/folder-open' }),
        clickCallback: () => void openFolder(),
      },
    ],
  },
])
</script>
<template><SideNavLayout :side-nav-groups="groups" /></template>
