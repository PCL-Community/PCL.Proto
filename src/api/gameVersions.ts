import sideTip from "@/composables/sideTip"
import { LocalStorageKeys } from "@/util/localStorage"
import { ref } from "vue"

const versionManifestUrl = "https://launchermeta.mojang.com/mc/game/version_manifest.json"
export type gameVersionType = 'snapshot' | 'release' | 'old_beta' | 'old_alpha'


interface IVersionManifest {
    latest: {
        release: string,
        snapshot: string
    },
    versions: {
        id: string,
        type: gameVersionType,
        url: string,
        time: string,
        releaseTime: string
    }[]
}

interface SingleVersionShowInfo {
    id: string,
    releaseTime: string,
}

export interface IVersionShow {
    latest: {
        release: SingleVersionShowInfo,
        snapshot: SingleVersionShowInfo,
    },
    release: SingleVersionShowInfo[],
    snapshot: SingleVersionShowInfo[],
    old: SingleVersionShowInfo[],
}

// 获取Minecraft版本信息
export function getMinecraftVersions() {
    // 1. 先尝试读取缓存
    const cacheStr = localStorage.getItem(LocalStorageKeys.MinecraftVersionManifest);
    let cacheVersionData: IVersionShow | undefined = undefined;
    const versionData = ref<IVersionShow>()
    if (cacheStr) {
        try {
            const cache = JSON.parse(cacheStr);
            cacheVersionData = cache.data as IVersionShow;
            versionData.value = cacheVersionData;
        } catch (e) {
            // 缓存解析失败，忽略
        }
    }

    // 2. 异步请求远程数据，获取后自动更新缓存和versionData
    fetch(versionManifestUrl)
        .then(res => res.json())
        .then((data: IVersionManifest) => {
            const latestRelease = data.versions.find(v => v.id === data.latest.release)!;
            const latestSnapshot = data.versions.find(v => v.id === data.latest.snapshot)!;
            const releaseVersions = data.versions.filter(v => v.type === 'release');
            const snapshotVersions = data.versions.filter(v => v.type === 'snapshot');
            const oldVersions = data.versions.filter(v =>
                v.type === 'old_beta' || v.type === 'old_alpha'
            );
            const result: IVersionShow = {
                latest: {
                    release: mapVersionToShow(latestRelease),
                    snapshot: mapVersionToShow(latestSnapshot)
                },
                release: releaseVersions.map(mapVersionToShow),
                snapshot: snapshotVersions.map(mapVersionToShow),
                old: oldVersions.map(mapVersionToShow)
            };
            // 更新缓存和响应式数据
            localStorage.setItem(LocalStorageKeys.MinecraftVersionManifest, JSON.stringify({
                time: Date.now(),
                data: result
            }));

            // 内容比较而非引用比较
            if (JSON.stringify(cacheVersionData) !== JSON.stringify(result)) {
                versionData.value = result;
                console.log(versionData.value.latest.release, result.latest.release)
                sideTip.show('游戏版本信息已刷新')
            }
        });

    function formatReleaseTime(iso: string) {
        const date = new Date(iso)
        const pad = (n: number) => n.toString().padStart(2, '0')
        return `${date.getFullYear()}/${pad(date.getMonth() + 1)}/${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`
    }

    function mapVersionToShow(v: { id: string; releaseTime: string; type: gameVersionType }) {
        return {
            id: v.id,
            releaseTime: formatReleaseTime(v.releaseTime),
        };
    }

    // 3. 返回缓存内容（同步）和响应式数据（异步）
    return { cacheVersionData, versionDataRef: versionData };
}