import { pclText } from '@/api/pclLocale'
import type { McProfile } from '@/stores/account'

/** 与 CE 的 ProfileUi.GetProfileInfo 保持一致。 */
export function getProfileTypeLabel(profile?: McProfile | null): string {
  if (!profile) return pclText('Launch.Account.Type.Offline')
  switch (profile.profileType) {
    case 'microsoft':
      return pclText('Launch.Account.Type.Microsoft')
    case 'authlib':
      return `${pclText('Launch.Account.Type.ThirdParty')}${profile.serverName ? ` / ${profile.serverName}` : ''}`
    case 'yggdrasilConnect':
      return `${pclText('Launch.Account.Type.ThirdParty')}${profile.serverName ? ` / ${profile.serverName}` : ' / Yggdrasil Connect'}`
    default:
      return pclText('Launch.Account.Type.Offline')
  }
}

export function getProfileInfo(profile?: McProfile | null): string {
  const label = getProfileTypeLabel(profile)
  return profile?.description ? `${label}，${profile.description}` : label
}

/**
 * 第三方认证服务器对应的网站根地址。
 *
 * 对照 CE：`Server.BeforeFirst("api/yggdrasil/authserver")`，没有该段时退回去掉
 * `/authserver` 的 API 根地址。
 */
export function authSiteRoot(server?: string | null): string | undefined {
  if (!server) return undefined
  const marker = server.indexOf('api/yggdrasil')
  const root = marker > 0 ? server.slice(0, marker) : server.replace(/\/authserver\/?$/, '')
  if (!root) return undefined
  return root.endsWith('/') ? root : `${root}/`
}

/** 皮肤站的衣柜页面，对照 CE 的 `user/closet`。 */
export function authClosetUrl(server?: string | null): string | undefined {
  const root = authSiteRoot(server)
  return root ? `${root}user/closet` : undefined
}

/** 皮肤站的账号设置页面，对照 CE 的 `user/profile`。 */
export function authAccountUrl(server?: string | null): string | undefined {
  const root = authSiteRoot(server)
  return root ? `${root}user/profile` : undefined
}
