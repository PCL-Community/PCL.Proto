type AvatarInputType = 'url' | 'username' | 'uuid'
import { invoke } from "@tauri-apps/api/core"
import SteveSkin from "/default-skin/Steve_(classic_texture)_JE6.png"

export default async function getSkinUrl(src: string, type?: AvatarInputType) {
    switch (type) {
        case 'username':
            const uuid = await invoke<string>('fetch_username_uuid', { username: src })
            return await getSkinUrl(uuid, 'uuid')
        case 'uuid':
            const session_data = await invoke<{ id: string, name: string, profileActions: [], properties: [{ name: string, value: string }] }>('fetch_uuid_profile', { uuid: src })
            console.log('session_data', session_data)
            const textures: { profileId: string, profileName: string, textures: { SKIN: { url: string }, CAPE?: { url: string } }, timestamp: number } = JSON.parse(atob(session_data.properties[0].value))
            console.log('textures', textures)
            return textures.textures.SKIN.url
        case 'url':
            return src
        default:
            return SteveSkin
    }
}