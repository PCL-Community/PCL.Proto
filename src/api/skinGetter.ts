type AvatarInputType = 'username' | 'uuid'
import { invoke } from '@tauri-apps/api/core'

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
        const skinUrl = await invoke<string | null>('fetch_skin_url', { uuid: src })
        if (!skinUrl) {
          onUpdate(`${import.meta.env.BASE_URL}pcl/images/Skins/Steve.png`)
          break
        }
        try {
          const base64Skin = await invoke<string>('fetch_skin_from_url', {
            url: skinUrl,
            uuid: src,
          })
          onUpdate(base64Skin)
          break
        } catch (err) {
          console.warn('Failed to fetch base64-skin from rust side, falling back to raw url.')
          onUpdate(skinUrl)
          break
        }
      }
    default:
      onUpdate(`${import.meta.env.BASE_URL}pcl/images/Skins/Steve.png`)
  }
}
