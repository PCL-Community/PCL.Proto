import IconFavorites from '@/components/icons/side/IconFavorites.vue'
import IconFourLeaves from '@/components/icons/side/IconFourLeaves.vue'
import IconMod from '@/components/icons/side/IconMod.vue'
import IconOverview from '@/components/icons/side/IconOverview.vue'
import IconPack from '@/components/icons/side/IconPack.vue'
import IconPicture from '@/components/icons/side/IconPicture.vue'
import IconSun from '@/components/icons/side/IconSun.vue'
import IconWrench from '@/components/icons/side/IconWrench.vue'
import SideNavLayout from '@/layout/SideNavLayout.vue'
import { defineComponent, h } from 'vue'
import { useI18n } from 'vue-i18n'

export default defineComponent({
  setup() {
    const { t } = useI18n()

    return () =>
      h(SideNavLayout, {
        sideNavGroups: [
          {
            title: t('download.nav.minecraft'),
            content: [
              {
                text: t('download.nav.game_download'),
                icon: IconOverview,
                linkto: '/download/game',
              },
              {
                text: t('download.nav.manual_install'),
                icon: IconWrench,
                linkto: '/download/manual',
              },
            ],
          },
          {
            title: t('download.nav.community_resource'),
            content: [
              { text: t('download.nav.mod'), icon: IconMod, linkto: '/download/mod' },
              { text: t('download.nav.mod_pack'), icon: IconPack, linkto: '/download/modpack' },
              { text: t('download.nav.data_pack'), icon: IconFourLeaves },
              {
                text: t('download.nav.resource_pack'),
                icon: IconPicture,
                linkto: '/download/resourcepack',
              },
              { text: t('download.nav.shader_pack'), icon: IconSun, linkto: '/download/shader' },
              { text: t('download.nav.favorites'), icon: IconFavorites },
            ],
          },
        ],
      })
  },
})
