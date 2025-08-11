import CardInfoItem from '@/components/widget/CardInfoItem.vue'
import PCard from '@/components/widget/PCard.vue'
import { pluginShowText, showIconPath } from '@/util/gameInfo'
import { useSelectedInstance } from '@/stores/gameLaunch'
import { computed, defineComponent } from 'vue'

const OverviewCard = defineComponent({
  name: 'OverviewCard',
  setup() {
    const selectedInstance = useSelectedInstance()
    const plugin = computed(() => selectedInstance.plugins[0])
    return () => (
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
  },
})

export default OverviewCard
