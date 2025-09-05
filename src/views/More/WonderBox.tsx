import MinecraftServerCard from '@/components/widget/MinecraftServerCard.vue'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import { useModal } from '@/composables/useModal'
import { defineAsyncComponent, defineComponent } from 'vue'
import { useAccountInfo } from '@/stores/account'
import PLoading from '@/components/widget/PLoading.vue'
import { RouterLink } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
const modal = useModal()

// const stringToRandom = (input: string) => {
//   let hash = input.charCodeAt(input.length - 1)
//   for (let i = 0; i < input.length; i++) {
//     hash = (hash << 5) + hash + input.charCodeAt(i)
//   }
//   return (hash % 101) + 1
// }
// 功能迁移至后端
const getRating = (luckValue: number): string => {
  switch (true) {
    case luckValue == 100:
      return '100！100！\n隐藏主题 欧皇…… 不对，Proto版应该没有这玩意……'
    case luckValue >= 95:
      return '差一点就到100了呢...'
    case luckValue >= 90:
      return '好评如潮！'
    case luckValue >= 60:
      return '还行啦，还行啦'
    case luckValue >= 40:
      return '勉强还行吧...'
    case luckValue >= 30:
      return '呜...'
    case luckValue >= 10:
      return '不会吧！'
    default:
      return '（是百分制哦）'
  }
}

export const LuckTodayButton = () => (
  <PButton
    click={() => {
      const date = new Date()
      const dateStr = date.toLocaleDateString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
      })
      invoke('get_lucky_today').then((luckValue) => {
        modal.open({
          title: `今日人品 - ${dateStr}`,
          content: `你今天的人品值是：${luckValue}… ${getRating(luckValue as number)}`,
          buttons: [{ type: 'default', content: '确定' }],
        })
      })
      // const luckValue = stringToRandom(dateStr)
      // 应该根据日期设置随机数种子，使当天得到的结果相同
    }}
  >
    今日人品
  </PButton>
)

export default defineComponent({
  name: 'WonderBox',
  setup() {
    const accountInfo = useAccountInfo()
    const SkinViewer = defineAsyncComponent({
      loader: () => import('@/components/widget/SkinViewer.vue'),
      loadingComponent: PLoading,
    })
    return () => (
      <>
        <PCard title="百宝箱">
          <div class="button-grid">
            <LuckTodayButton />
            <RouterLink to="/homepageeditor">
              <PButton style={{ width: '100%' }}>首页设计工具</PButton>
            </RouterLink>
          </div>
        </PCard>
        <PCard title="下载正版玩家的皮肤">
          <SkinViewer skinUrl={accountInfo.skinUrl} />
        </PCard>
        <MinecraftServerCard />
      </>
    )
  },
})
