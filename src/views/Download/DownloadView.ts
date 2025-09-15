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
      { text: 'download.nav.game_download', icon: IconOverview, linkto: '/download/game' },
      { text: 'download.nav.manual_install', icon: IconWrench, linkto: "manual" }
    ]
  }, {
    title: 'download.nav.community_resource',
    content: [
      { text: 'download.nav.mod', icon: IconMod, linkto: 'mod' },
      { text: 'download.nav.mod_pack', icon: IconPack, linkto: 'modpack' },
      { text: 'download.nav.data_pack', icon: IconFourLeaves },
      { text: 'download.nav.resource_pack', icon: IconPicture, linkto: 'resourcepack' },
      { text: 'download.nav.shader_pack', icon: IconSun, linkto: 'shader' },
      { text: 'download.nav.favorites', icon: IconFavorites }
    ]
  }
  ]
})
