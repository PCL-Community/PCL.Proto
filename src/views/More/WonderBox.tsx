import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import { useModal } from '@/composables/useModal'
const modal = useModal()

export default () => (
  <>
    <PCard title="百宝箱">
      <div class="button-grid">
        <PButton
          click={() => {
            modal.open({
              title: '今日人品 - 2025/08/07',
              content: '你今天的人品值是：31… 呜…',
              buttons: [{ type: 'default', content: '确定' }],
            })
          }}
        >
          今日人品
        </PButton>
      </div>
    </PCard>
    <PCard title="下载正版玩家的皮肤"></PCard>
    <PCard title="瞅眼服务器"></PCard>
  </>
)
