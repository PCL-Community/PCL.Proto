import IconSetup from '@/components/icons/header/IconSetup.vue'
import IconFeedback from '@/components/icons/side/IconFeedback.vue'
import IconHelp from '@/components/icons/side/IconHelp.vue'
import IconLobby from '@/components/icons/side/IconLobby.vue'
import SideNavLayout from '@/layout/SideNavLayout.vue'
import { defineComponent, h } from 'vue'

export default defineComponent({
  setup() {
    return () =>
      h(SideNavLayout, {
        sideNavGroups: [{
          title: '开始联机',
          content: [
            { text: '大厅', icon: IconLobby, linkto: 'lobby' },
          ]
        }, {
          title: "更多选项",
          content: [
            { text: '设置', icon: IconSetup, linkto: 'setup' },
            { text: '常见问题', icon: IconHelp, linkto: 'help' },
            { text: '关于和反馈', icon: IconFeedback, linkto: 'feedback' }
          ]
        }
        ]
      })
  },
})
