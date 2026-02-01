import IconInfo from '@/components/icons/control/IconInfo.vue'
import IconLaunch from '@/components/icons/header/IconLaunch.vue'
import IconLink from '@/components/icons/header/IconLink.vue'
import IconFeedback from '@/components/icons/side/IconFeedback.vue'
import IconJava from '@/components/icons/side/IconJava.vue'
import IconPack from '@/components/icons/side/IconPack.vue'
import IconPaint from '@/components/icons/side/IconPaint.vue'
import SideNavLayout from '@/layout/SideNavLayout.vue'
import { h } from 'vue'
import { useI18n } from 'vue-i18n'

export default {
  setup() {
    const { t } = useI18n()
    return () =>
      h(SideNavLayout, {
        sideNavGroups: [
          {
            title: t('setup.nav.game'),
            content: [
              { text: '启动', icon: IconLaunch, linkto: 'launch' },
              { text: 'Java管理', icon: IconJava, linkto: 'java' },
            ],
          },
          {
            title: t('setup.nav.launcher'),
            content: [
              { text: '个性化', icon: IconPaint, linkto: 'personalization' },
              { text: '其他', icon: IconPack },
              { text: '联机', icon: IconLink },
            ],
          },
          {
            title: t('setup.nav.info'),
            content: [
              { text: t('setup.nav.about'), icon: IconInfo, linkto: 'about' },
              { text: t('setup.nav.feedback'), icon: IconFeedback },
            ],
          }
        ],
      })
  },
}
