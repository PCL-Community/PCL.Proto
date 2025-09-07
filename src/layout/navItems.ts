import IconLaunch from "@/components/icons/header/IconLaunch.vue"
import IconDownload from "@/components/icons/header/IconDownload.vue"
import IconLink from "@/components/icons/header/IconLink.vue"
import IconSetup from "@/components/icons/header/IconSetup.vue"
import IconMore from "@/components/icons/header/IconMore.vue"

const navItems = [
    { to: '/home', icon: IconLaunch, label: 'main_nav.home' },
    { to: '/download', icon: IconDownload, label: 'main_nav.download' },
    { to: '/link', icon: IconLink, label: 'main_nav.link' },
    { to: '/setup', icon: IconSetup, label: 'main_nav.setup' },
    { to: '/more', icon: IconMore, label: 'main_nav.more' },
]

export default navItems 