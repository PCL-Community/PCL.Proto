import CardInfoItem from '@/components/widget/CardInfoItem.vue'
import PCard from '@/components/widget/PCard.vue'
import { pluginShowText, showIconPath } from '@/util/gameInfo'
import { selectedInstance } from '@/util/gameLaunch'
import { computed } from 'vue'

const plugin = computed(() => selectedInstance.plugins[0])

function OverviewCard() {
  return (
    <PCard hideTitle={true}>
      {{
        content: () => (
          <CardInfoItem
            title={selectedInstance.name}
            subtitle={`${selectedInstance.version}, ${pluginShowText[plugin.value]} ${selectedInstance.pluginsVersion[plugin.value]}`}
            icon={showIconPath[plugin.value]}
            hoverEffect={false}
          ></CardInfoItem>
        ),
      }}
    </PCard>
  )
}

export default OverviewCard
