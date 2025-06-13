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

export async function getMinecraftVersions(): Promise<IVersionShow> {
    const res = await fetch(versionManifestUrl)
    const data: IVersionManifest = await res.json()

    // 查找最新release和snapshot的完整信息
    const latestRelease = data.versions.find(v => v.id === data.latest.release)!;
    const latestSnapshot = data.versions.find(v => v.id === data.latest.snapshot)!;

    // 按照类型分类版本
    const releaseVersions = data.versions.filter(v => v.type === 'release');
    const snapshotVersions = data.versions.filter(v => v.type === 'snapshot');
    const oldVersions = data.versions.filter(v =>
        v.type === 'old_beta' || v.type === 'old_alpha'
    );

    return {
        latest: {
            release: mapVersionToShow(latestRelease),
            snapshot: mapVersionToShow(latestSnapshot)
        },
        release: releaseVersions.map(mapVersionToShow),
        snapshot: snapshotVersions.map(mapVersionToShow),
        old: oldVersions.map(mapVersionToShow)
    };

    function mapVersionToShow(v: { id: string; releaseTime: string; type: gameVersionType }) {
        return {
            id: v.id,
            releaseTime: v.releaseTime,
        };
    }
}

export const versionData = ref<IVersionShow>();

// getMinecraftVersions().then((data) => {
//     versionData.value = data;
//     let latestIcon = showIconPath[gameInfoIcon[data.latest.release.type]]
//     console.log(latestIcon)
// })