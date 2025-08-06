import IconButtonSave from '@/components/icons/control/IconButtonSave.vue'
import IconCommand from '@/components/icons/side/IconCommand.vue'
import IconMod from '@/components/icons/side/IconMod.vue'
import IconOverview from '@/components/icons/side/IconOverview.vue'
import IconPack from '@/components/icons/side/IconPack.vue'
import IconPicture from '@/components/icons/side/IconPicture.vue'
import IconSchematic from '@/components/icons/side/IconSchematic.vue'
import IconSun from '@/components/icons/side/IconSun.vue'
import IconWrench from '@/components/icons/side/IconWrench.vue'
import CardInfoItem from '@/components/widget/CardInfoItem.vue'
import PCard from '@/components/widget/PCard.vue'
import SideNavLayout from '@/layout/SideNavLayout.vue'
import { showIconPath, type showIconType } from '@/util/gameInfo'
import { selectedInstance } from '@/util/gameLaunch'
import { computed, h } from 'vue'

// 实例信息展示
export const cardInfo = computed(() => {
  return {
    title: selectedInstance.name,
    subtitle: `${selectedInstance.version}, ${selectedInstance.modLoaderInfo}`,
    icon: showIconPath[selectedInstance.modLoader as showIconType],
    hoverEffect: false,
  }
})

export const OverviewCard = h(PCard, { hideTitle: true }, { content: () => h(CardInfoItem, cardInfo.value) })

export default {
  setup() {
    return () =>
      h(SideNavLayout, {
        sideNavGroups: [
          {
            title: '游戏本体',
            content: [
              { text: '概览', linkto: 'overview', icon: IconOverview },
              { text: '设置', linkto: 'setting', icon: IconWrench },
              { text: '修改', linkto: 'modify', icon: IconCommand },
              { text: '导出', linkto: 'export', icon: IconPack },
            ],
          },
          {
            title: '游戏资源',
            content: [
              { text: '存档', linkto: 'save', icon: IconButtonSave },
              { text: '截图', linkto: 'screenshot', icon: IconPicture },
              { text: 'Mod', linkto: 'mod', icon: IconMod },
              { text: '资源包', linkto: 'resourcepack', icon: IconPicture },
              { text: '光影包', linkto: 'shaderpack', icon: IconSun },
              { text: '投影原理图', linkto: 'projection', icon: IconSchematic },
            ],
          },
        ],
      })
  },
}
