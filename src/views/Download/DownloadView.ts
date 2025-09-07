import IconFavorites from '@/components/icons/side/IconFavorites.vue';
import IconFourLeaves from '@/components/icons/side/IconFourLeaves.vue';
import IconMod from '@/components/icons/side/IconMod.vue';
import IconOverview from '@/components/icons/side/IconOverview.vue';
import IconPack from '@/components/icons/side/IconPack.vue';
import IconPicture from '@/components/icons/side/IconPicture.vue';
import IconSun from '@/components/icons/side/IconSun.vue';
import IconWrench from '@/components/icons/side/IconWrench.vue';
import SideNavLayout from '@/layout/SideNavLayout.vue'
import { h } from 'vue';

export default h(SideNavLayout, {
  sideNavGroups: [{
    title: 'download.nav.minecraft',
    content: [
      { text: 'download.nav.game_download', icon: IconOverview, linkto: 'game' },
      { text: 'download.nav.manual_install', icon: IconWrench, linkto: "manual" }
    ]
  }, {
    title: 'download.nav.community_resource',
    content: [
      { text: 'download.nav.mod', icon: IconMod, linkto: 'mod' },
      { text: '整合包', icon: IconPack, linkto: 'modpack' },
      { text: '数据包', icon: IconFourLeaves },
      { text: '资源包', icon: IconPicture, linkto: 'resourcepack' },
      { text: '光影包', icon: IconSun, linkto: 'shader' },
      { text: '收藏夹', icon: IconFavorites }
    ]
  }
  ]
})
