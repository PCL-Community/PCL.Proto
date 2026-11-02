import { useModal } from '@/composables/useModal'
import { pclText } from '@/api/pclLocale'
import sideTip from '@/composables/sideTip'
import { openExternal } from '@/util/externalLink'

/**
 * 设备码登录弹窗，对照 CE 的 `MyMsgLogin`：重新打开网页、复制代码、取消。
 *
 * 轮询在后端进行，取消时调用 `onCancel` 让后端停下。
 */
export function PMsgLogin(
  code: string,
  verificationUrl: string,
  caption: string,
  onCancel: () => void,
) {
  const modal = useModal()
  return modal.open({
    title: pclText('Launch.Account.LoginDialog.MinecraftLogin'),
    content: caption,
    buttons: [
      {
        content: pclText('Launch.Account.LoginDialog.ReopenWebpage'),
        operation: () => void openExternal(verificationUrl),
      },
      {
        content: pclText('Launch.Account.LoginDialog.CopyCode'),
        operation: async () => {
          try {
            await navigator.clipboard.writeText(code)
            sideTip.show(pclText('Common.Hint.Copied'), 'success')
          } catch {
            sideTip.show(pclText('Proto.Clipboard.CopyFailed'), 'warn')
          }
        },
      },
      {
        content: pclText('Common.Action.Cancel'),
        operation: () => {
          onCancel()
          modal.close(false)
        },
      },
    ],
  })
}
