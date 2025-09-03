import IconLaunch from "@/components/icons/header/IconLaunch.vue"
import IconDownload from "@/components/icons/header/IconDownload.vue"
import IconLink from "@/components/icons/header/IconLink.vue"
import IconSetup from "@/components/icons/header/IconSetup.vue"
import IconMore from "@/components/icons/header/IconMore.vue"

const navItems = [
    { to: '/home', icon: IconLaunch, label: '启动' },
    { to: '/download', icon: IconDownload, label: '下载' },
    { to: '/link', icon: IconLink, label: '联机' },
    { to: '/setup', icon: IconSetup, label: '设置' },
    { to: '/more', icon: IconMore, label: '更多' },
]

export default navItems 