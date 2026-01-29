import IconHelp from '@/components/icons/side/IconHelp.vue'
import IconLobby from '@/components/icons/side/IconLobby.vue'
import IconPack from '@/components/icons/side/IconPack.vue'
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
            title: t('tools.nav.link'),
            content: [{ text: t('tools.nav.lobby'), icon: IconLobby, linkto: 'lobby' }],
          },
          {
            title: t('tools.nav.wondertoys'),
            content: [
              { text: t('tools.nav.wonderbox'), icon: IconPack, linkto: 'wonderbox' },
              { text: t('tools.nav.help'), icon: IconHelp, linkto: 'help' },
            ],
          },
        ],
      })
  },
})
