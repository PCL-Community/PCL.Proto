import IconOverview from "@/components/icons/side/IconOverview.vue"
import IconWrench from "@/components/icons/side/IconWrench.vue"
import SideNavLayout from "@/layout/SideNavLayout.vue"
import { h } from "vue"

export default {
    setup() {
        return () => h(SideNavLayout, {
            sideNavGroups: [{
                title: '游戏本体',
                content: [
                    { text: '概览', linkto: 'overview', icon: IconOverview },
                    { text: '设置', linkto: 'setting', icon: IconWrench },
                    { text: '修改', linkto: 'modify' },
                    { text: '导出', linkto: 'export' }
                ]
            }, {
                title: '游戏资源',
                content: [
                    { text: '存档', linkto: 'save' },
                    { text: '截图', linkto: 'screenshot' },
                    { text: 'Mod', linkto: 'mod' },
                    { text: '资源包', linkto: 'resourcepack' },
                    { text: '光影包', linkto: 'shaderpack' },
                    { text: '投影原理图', linkto: 'projection' },
                ]
            }]
        })
    }
}