import IconInfo from '@/components/icons/control/IconInfo.vue';
import IconFeedback from '@/components/icons/side/IconFeedback.vue';
import IconHelp from '@/components/icons/side/IconHelp.vue';
import IconPack from '@/components/icons/side/IconPack.vue';
import IconVote from '@/components/icons/side/IconVote.vue';
import SideNavLayout from '@/layout/SideNavLayout.vue';
import { h } from 'vue';

export default h(SideNavLayout, {
    sideNavGroups: [{
        title: '更多',
        content: [
            { text: '帮助', icon: IconHelp },
            { text: '关于与鸣谢', icon: IconInfo, linkto: 'about_and_thanks' },
            { text: '百宝箱', icon: IconPack, linkto: 'wonder_box' },
            { text: '新功能投票', icon: IconVote },
            { text: '反馈', icon: IconFeedback }
        ]
    }
    ]
})
