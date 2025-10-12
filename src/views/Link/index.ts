import IconSetup from '@/components/icons/header/IconSetup.vue'
import IconFeedback from '@/components/icons/side/IconFeedback.vue'
import IconHelp from '@/components/icons/side/IconHelp.vue'
import IconLobby from '@/components/icons/side/IconLobby.vue'
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
            title: t('link.nav.start'),
            content: [{ text: t('link.nav.lobby'), icon: IconLobby, linkto: 'lobby' }],
          },
          {
            title: t('link.nav.more_options'),
            content: [
              { text: t('link.nav.setting'), icon: IconSetup, linkto: 'setup' },
              { text: t('link.nav.common_question'), icon: IconHelp, linkto: 'help' },
              { text: t('link.nav.about_and_feedback'), icon: IconFeedback, linkto: 'feedback' },
            ],
          },
        ],
      })
  },
})
