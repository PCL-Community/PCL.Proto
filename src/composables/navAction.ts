import { inject, onUnmounted, type InjectionKey } from 'vue'

export interface NavActionRegistry {
  register: (kind: string, handler: () => unknown) => () => void
}

export const NAV_ACTION_REGISTRY: InjectionKey<NavActionRegistry> = Symbol('navActionRegistry')

/**
 * 页面注册左侧导航按钮（刷新 / 初始化）的实际行为。
 * 未注册时 SideNavLayout 保留默认行为（刷新重挂载页面、初始化弹未实现提示）。
 */
export function useNavAction(kind: string, handler: () => unknown) {
  const registry = inject<NavActionRegistry | null>(NAV_ACTION_REGISTRY, null)
  if (!registry) return
  onUnmounted(registry.register(kind, handler))
}
