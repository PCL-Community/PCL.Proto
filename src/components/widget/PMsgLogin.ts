import { useModal } from '@/composables/useModal'
import { pclText } from '@/api/pclLocale'
import sideTip from '@/composables/sideTip'
import { openExternal } from '@/util/externalLink'

// Device-code login uses the existing application modal.
export function PMsgLogin(code: string, verificationUrl: string, caption: string) {
  const modal = useModal()
  return modal.open({
    title: pclText('Launch.Account.LoginDialog.Title'),
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
      { content: pclText('Common.Action.Cancel'), operation: () => modal.close(false) },
    ],
  })
}
