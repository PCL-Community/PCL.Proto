import IconLaunch from '@/components/icons/header/IconLaunch.vue';
import IconJava from '@/components/icons/side/IconJava.vue';
import IconPack from '@/components/icons/side/IconPack.vue';
import IconPaint from '@/components/icons/side/IconPaint.vue';
import SideNavLayout from '@/layout/SideNavLayout.vue';
import type { INavItem } from '@/types/naviOptions';
import { defineComponent, h } from 'vue';

export default defineComponent({
    setup() {
        return () =>
            h(SideNavLayout, {
                sideNavGroups: [{
                    content: [
                        { text: '启动', icon: IconLaunch, linkto: 'launch' },
                        { text: '个性化', icon: IconPaint },
                        { text: '其他', icon: IconPack },
                        { text: 'Java管理', icon: IconJava, linkto: 'java' }
                    ] as INavItem[]
                }
                ]
            })
    },
})