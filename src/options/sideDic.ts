import IconLaunch from "@/components/icons/header/IconLaunch.vue"
import IconDownload from "@/components/icons/header/IconDownload.vue"
import IconLink from "@/components/icons/header/IconLink.vue"
import IconSetup from "@/components/icons/header/IconSetup.vue"
import IconMore from "@/components/icons/header/IconMore.vue"

const navItems = [
    { to: '/home', icon: IconLaunch, label: '启动', width: 286 },
    { to: '/download', icon: IconDownload, label: '下载', width: 139 },
    { to: '/link', icon: IconLink, label: '联机', width: 120 },
    { to: '/setup', icon: IconSetup, label: '设置', width: 114 },
    { to: '/more', icon: IconMore, label: '更多', width: 120 },
]

export default navItems 