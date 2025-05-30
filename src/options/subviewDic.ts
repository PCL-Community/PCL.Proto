import IconOverview from "@/components/icons/side/IconOverview.vue";
import GameDownload from "@/views/DownloadSubView/GameDownload.vue";
import ManualDownload from "@/views/DownloadSubView/ManualDownload.vue";

interface SubData {
    subview: typeof IconOverview,
    text: string,
    icon: typeof IconOverview
}
const downloadSubViewList: SubData[] = [
    { subview: GameDownload, text: '游戏下载', icon: IconOverview },
    { subview: ManualDownload, text: '手动安装包', icon: IconOverview }
]

const viewDic = {
    'download': downloadSubViewList
}

export default viewDic