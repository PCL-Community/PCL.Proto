import IconButtonAdd from '@/components/icons/side/IconButtonAdd.vue'
import IconImportModpack from '@/components/icons/side/IconImportModpack.vue'
import SideNavLayout from '@/layout/SideNavLayout.vue'
import { useRepositoriesStore } from '@/stores/repositories'
import { defineComponent, onBeforeMount } from 'vue'
import { useRouter } from 'vue-router'

export default defineComponent({
  setup() {
    const repo = useRepositoriesStore()
    const router = useRouter()

    onBeforeMount(() => {
      if (repo.repositires.length > 0) {
        router.push('/instance_select/instance_select_sub/0')
      }
    })

    return () => (
      <SideNavLayout
        sideNavGroups={[
          {
            title: '文件夹列表',
            content:
              repo.repositires.map((item, index) => ({
                text: item.name,
                linkto: `/instance_select/instance_select_sub/${index}`,
              })) || [],
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
