import IconButtonAdd from '@/components/icons/side/IconButtonAdd.vue'
import IconImportModpack from '@/components/icons/side/IconImportModpack.vue'
import SideNavLayout from '@/layout/SideNavLayout.vue'
import { useRepositoriesStore } from '@/stores/repositories'
import { info } from '@tauri-apps/plugin-log'
import { defineComponent, onBeforeMount, onMounted } from 'vue'
import { useRouter } from 'vue-router'

export default defineComponent({
  setup() {
    const repo = useRepositoriesStore()
    const router = useRouter()

    onBeforeMount(async () => {
      await repo.fetchFromBackend()
    })

    onMounted(() => {
      repo
        .getInstancesInRepository('HMCL')
        .then((instances) => {
          info(`${instances.map((item) => item.name)}`)
        })
        .catch((err) => {
          info(`${err}`)
        })
    })

    return () => (
      <SideNavLayout
        sideNavGroups={[
          {
            title: '文件夹列表',
            // 此处需要动态加载
            // content: [{ text: '当前文件夹' }, { text: '官方启动器文件夹' }],
            content: repo.repositires.map((item) => ({ text: item.name, linkto: item.name })) || [],
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
