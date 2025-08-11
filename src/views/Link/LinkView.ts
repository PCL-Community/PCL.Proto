import IconSetup from '@/components/icons/header/IconSetup.vue';
import SideNavLayout from '@/layout/SideNavLayout.vue';
import { h } from 'vue';

export default h(SideNavLayout, {
  sideNavGroups: [{
    title: '开始联机',
    content: [
      { text: '大厅' },
    ]
  }, {
    title: "更多选项",
    content: [
      { text: '设置', icon: IconSetup },
      { text: '常见问题' },
      { text: '关于和反馈' }
    ]
  }
  ]
})
