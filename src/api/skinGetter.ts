type AvatarInputType = 'username' | 'uuid'
import { invoke } from '@tauri-apps/api/core'
import SteveSkin from '/default-skin/Steve_(classic_texture)_JE6.png'

// 获取皮肤URL，支持用户名、UUID、URL三种输入类型
// 当获取到本地皮肤URL后，会调用onUpdate回调函数，将新的URL作为参数传递
export default async function getSkinUrl(
  src: string,
  onUpdate: (newPath: string) => void,
  type?: AvatarInputType,
) {
  switch (type) {
    case 'username':
      const uuid = await invoke<string>('fetch_username_uuid', { username: src })
      await getSkinUrl(uuid, onUpdate, 'uuid')
      break
    case 'uuid':
      try {
        const base64SkinCached = await invoke<string>('fetch_skin_from_uuid_cached', { uuid: src })
        onUpdate(base64SkinCached)
        break
      } catch (err) {
        const session_data = await invoke<{
          id: string
          name: string
          profileActions: []
          properties: [{ name: string; value: string }]
        }>('fetch_uuid_profile', { uuid: src })
        console.debug('session_data', session_data)
        const textures: {
          profileId: string
          profileName: string
          textures: { SKIN: { url: string }; CAPE?: { url: string } }
          timestamp: number
        } = JSON.parse(atob(session_data.properties[0].value))
        console.debug('textures', textures)
        const skinUrl = textures.textures.SKIN.url
        const base64Skin = await invoke<string>('fetch_skin_from_url', { url: skinUrl, uuid: src })
        onUpdate(base64Skin)
        break
      }
    default:
      onUpdate(SteveSkin)
  }
}
