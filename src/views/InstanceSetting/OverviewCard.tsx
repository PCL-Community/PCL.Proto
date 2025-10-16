import CardInfoItem from '@/components/widget/CardInfoItem.vue'
import PCard from '@/components/widget/PCard.vue'
import { isIconPixelated, showIconPath } from '@/types/gameInfo'
import { useSelectedInstance } from '@/stores/gameLaunch'
import { computed, defineComponent } from 'vue'

const OverviewCard = defineComponent({
  name: 'OverviewCard',
  setup() {
    const { instance_info, plugins } = useSelectedInstance()

    if (instance_info != null) {
      const pluginsMap = instance_info.pluginsVersion
      const subtitle = computed(() => {
        let text = instance_info.version
        if (pluginsMap) {
          text += ` ${pluginsMap}`
        }
        return text
      })

      const iconType = computed(() => {
        if (pluginsMap && plugins[0]) {
          return plugins[0]
        } else {
          return 'vanilla'
        }
      })

      return () => (
        <PCard hideTitle={true}>
          <CardInfoItem
            title={instance_info.name}
            subtitle={subtitle.value}
            icon={showIconPath[iconType.value]}
            hoverEffect={false}
            pixelatedIcon={isIconPixelated[iconType.value]}
          ></CardInfoItem>
        </PCard>
      )
    } else {
      return () => 'error loading instance'
    }
  },
})

export default OverviewCard
