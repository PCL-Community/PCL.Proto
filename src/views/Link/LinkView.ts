import IconSetup from '@/components/icons/header/IconSetup.vue';
import IconFeedback from '@/components/icons/side/IconFeedback.vue';
import IconHelp from '@/components/icons/side/IconHelp.vue';
import IconLobby from '@/components/icons/side/IconLobby.vue';
import SideNavLayout from '@/layout/SideNavLayout.vue';
import { h } from 'vue';

export default h(SideNavLayout, {
  sideNavGroups: [{
    title: '开始联机',
    content: [
      { text: '大厅', icon: IconLobby },
    ]
  }, {
    title: "更多选项",
    content: [
      { text: '设置', icon: IconSetup },
      { text: '常见问题', icon: IconHelp },
      { text: '关于和反馈', icon: IconFeedback }
    ]
  }
  ]
})
