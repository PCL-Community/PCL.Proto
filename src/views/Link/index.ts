import IconSetup from '@/components/icons/header/IconSetup.vue';
import IconFeedback from '@/components/icons/side/IconFeedback.vue';
import IconHelp from '@/components/icons/side/IconHelp.vue';
import IconLobby from '@/components/icons/side/IconLobby.vue';
import SideNavLayout from '@/layout/SideNavLayout.vue';
import { h } from 'vue';

export default h(SideNavLayout, {
  sideNavGroups: [{
    title: 'link.nav.start',
    content: [
      { text: 'link.nav.lobby', icon: IconLobby, linkto: 'lobby' },
    ]
  }, {
    title: 'link.nav.more_options',
    content: [
      { text: 'link.nav.setting', icon: IconSetup, linkto: 'setup' },
      { text: 'link.nav.common_question', icon: IconHelp, linkto: 'help' },
      { text: 'link.nav.about_and_feedback', icon: IconFeedback, linkto: 'feedback' }
    ]
  }
  ]
})
