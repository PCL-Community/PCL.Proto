<script lang="ts">
import IconFourLeaves from '@/components/icons/side/IconFourLeaves.vue';
import IconMod from '@/components/icons/side/IconMod.vue';
import IconOverview from '@/components/icons/side/IconOverview.vue';
import IconPack from '@/components/icons/side/IconPack.vue';
import IconPicture from '@/components/icons/side/IconPicture.vue';
import IconSun from '@/components/icons/side/IconSun.vue';
import IconWrench from '@/components/icons/side/IconWrench.vue';
import SideNavLayout from '@/layout/SideNavLayout.vue'
import { h } from 'vue';


export default {
  setup() {
    return () => h(SideNavLayout, {
      sideNavGroups: [{
        title: 'Minecraft',
        content: [
          { itemName: '游戏下载', icon: IconOverview, linkto: 'game' },
          { itemName: '手动安装包', icon: IconWrench, linkto: "manual" }
        ]
      }, {
        title: '社区资源',
        content: [
          { itemName: 'Mod', icon: IconMod },
          { itemName: '整合包', icon: IconPack },
          { itemName: '数据包', icon: IconFourLeaves },
          { itemName: '资源包', icon: IconPicture },
          { itemName: '光影包', icon: IconSun }
        ]
      }
      ]
    })
  }
}
</script>