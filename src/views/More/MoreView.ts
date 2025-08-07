import IconInfo from '@/components/icons/control/IconInfo.vue';
import IconLaunch from '@/components/icons/header/IconLaunch.vue';
import IconPack from '@/components/icons/side/IconPack.vue';
import SideNavLayout from '@/layout/SideNavLayout.vue';
import { h } from 'vue';

export default {
    setup() {
        return () => h(SideNavLayout, {
            sideNavGroups: [{
                title: '更多',
                content: [
                    { text: '帮助', icon: IconLaunch },
                    { text: '关于与鸣谢', icon: IconInfo, linkto: 'about_and_thanks' },
                    { text: '百宝箱', icon: IconPack, linkto: 'wonder_box' },
                    { text: '反馈' }
                ]
            }
            ]
        })
    },
}
