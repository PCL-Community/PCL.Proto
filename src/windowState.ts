import { reactive } from "vue";

export const defaultWidths = {
    home: 286,
    download: 139,
    link: 120,
    setup: 120,
    more: 120
}

// 全局侧边栏状态管理
export const sideNavState = reactive({
    isShown: true,
    width: defaultWidths.home
})

