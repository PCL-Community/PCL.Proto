import { invoke } from "@tauri-apps/api/core"
import { ref } from "vue"

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
export function useMinecraftVersions() {
    const versionDataRef = ref<IVersionShow>()
    invoke<IVersionManifest>('get_version_manifest')
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
            versionDataRef.value = result;
        });

    function formatReleaseTime(iso: string) {
        const date = new Date(iso)
        return `${date.toLocaleString('zh-CN', {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
        })}`
    }

    function mapVersionToShow(v: { id: string; releaseTime: string; type: gameVersionType }) {
        return {
            id: v.id,
            releaseTime: formatReleaseTime(v.releaseTime),
        };
    }

    return versionDataRef;
}