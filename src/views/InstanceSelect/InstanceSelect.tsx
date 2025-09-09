import IconButtonAdd from '@/components/icons/side/IconButtonAdd.vue'
import IconImportModpack from '@/components/icons/side/IconImportModpack.vue'
import SideNavLayout from '@/layout/SideNavLayout.vue'
import { invoke } from '@tauri-apps/api/core'
import { defineComponent, onBeforeMount, ref } from 'vue'

export default defineComponent({
  setup() {
    const directories = ref<{ name: string; path: string }[]>()

    onBeforeMount(async () => {
      directories.value = await invoke('get_game_directories')
    })

    return () => (
      <SideNavLayout
        sideNavGroups={[
          {
            title: '文件夹列表',
            // 此处需要动态加载
            // content: [{ text: '当前文件夹' }, { text: '官方启动器文件夹' }],
            content: directories.value?.map((item) => ({ text: item.name })) || [],
          },
          {
            title: '添加或导入',
            content: [
              { text: '添加新文件夹', icon: IconButtonAdd },
              { text: '导入整合包', icon: IconImportModpack },
            ],
          },
        ]}
      />
    )
  },
})
