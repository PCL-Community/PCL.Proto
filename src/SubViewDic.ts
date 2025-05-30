import GameDownload from "./views/DownloadSubView/GameDownload.vue";
import ModDownload from "./views/DownloadSubView/ModDownload.vue";
import HomeSubView from "./views/HomeSubView.vue";

const downloadSubViewList = [
    GameDownload,
    ModDownload,
    HomeSubView
]

const viewDic = {
    'download': downloadSubViewList
}

export default viewDic