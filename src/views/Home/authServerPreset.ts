import { ref } from 'vue'

/**
 * 新建档案时选好的第三方认证服务器，对照 CE 的 `PageLoginAuth.draggedAuthServer`。
 *
 * 档案列表页写入，第三方登录页读取后清空。
 */
export const authServerPreset = ref<string | null>(null)
