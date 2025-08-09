import MinecraftServerCard from '@/components/widget/MinecraftServerCard.vue'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PInput from '@/components/widget/PInput.vue'
import { useModal } from '@/composables/useModal'
import { defineComponent, ref } from 'vue'
const modal = useModal()

const stringToRandom = (input: string) => {
  let hash = input.charCodeAt(input.length - 1)
  for (let i = 0; i < input.length; i++) {
    hash = (hash << 5) + hash + input.charCodeAt(i)
  }
  return (hash % 101) + 1
}

export const LuckToday = () => (
  <PButton
    click={() => {
      const date = new Date()
      const dateStr = date.toLocaleDateString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
      })
      const luckValue = stringToRandom(dateStr)
      // 应该根据日期设置随机数种子，使当天得到的结果相同
      modal.open({
        title: `今日人品 - ${dateStr}`,
        content: `你今天的人品值是：${luckValue}… 呜…`,
        buttons: [{ type: 'default', content: '确定' }],
      })
    }}
  >
    今日人品
  </PButton>
)

export default defineComponent({
  name: 'WonderBox',
  setup() {
    const serverInput = ref<string>()
    const minecraftServerCardVisible = ref<boolean>(false)
    return () => (
      <>
        <PCard title="百宝箱">
          <div class="button-grid">
            <LuckToday />
          </div>
        </PCard>
        <PCard title="下载正版玩家的皮肤"></PCard>
        <PCard title="瞅眼服务器">
          <div
            style={{
              display: 'flex',
              gap: '16px',
              alignItems: 'center',
              justifyContent: 'space-around',
              marginBottom: '8px',
            }}
          >
            <PInput
              v-model={serverInput.value}
              placeholder="输入服务器地址"
              style={{ flex: '1' }}
            />
            <PButton
              inline
              click={() => {
                minecraftServerCardVisible.value = true
              }}
            >
              查询
            </PButton>
          </div>
          {minecraftServerCardVisible.value && serverInput.value && (
            <MinecraftServerCard url={serverInput.value} />
          )}
        </PCard>
      </>
    )
  },
})
