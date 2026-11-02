import { computed, watch } from 'vue'
import type { ComputedRef } from 'vue'
import { pclText } from '@/api/pclLocale'
import sideTip from '@/composables/sideTip'
import { useModal } from '@/composables/useModal'
import { PMsgLogin } from '@/components/widget/PMsgLogin'
import { ModalWidthVariant } from '@/types/modal'
import { reportLoginFailure } from '@/util/loginErrors'
import { openExternal } from '@/util/externalLink'
import { CANCELLED, useAccountInfo } from '@/stores/account'

/** CE 弹窗打开 2 秒后自动打开网页并复制授权码。 */
const AUTO_OPEN_DELAY = 2000

/** 登录按钮上的百分比文本，对照 CE 的 `Lang.Number(progress, "P0")`。 */
export function loginProgress(context: 'microsoft' | 'auth'): ComputedRef<string | null> {
  const account = useAccountInfo()
  return computed(() => {
    if (!account.loggingIn) return null
    return account.loginState.status === 'succeeded' ? '98%' : context === 'auth' ? '10%' : '5%'
  })
}

/**
 * 驱动一次登录会话的界面部分：设备码弹窗、角色选择弹窗与结果提示。
 *
 * 对照 CE 的 `MyMsgLogin` 弹窗 + `ProfileUi.ShowDeviceCodeLoginAsync` +
 * `_SelectAuthProfile`。必须挂在登录期间不会卸载的组件上（启动页），
 * 否则用户在登录途中切页会让弹窗与结果提示丢失。
 */
export function useLoginSession(context: () => 'microsoft' | 'auth', onSucceeded?: () => void) {
  const account = useAccountInfo()
  const modal = useModal()
  let dialogShown = false

  async function copyCode(code: string) {
    try {
      await navigator.clipboard.writeText(code)
    } catch (error) {
      console.warn('Failed to copy the device code', error)
    }
  }

  async function showDeviceCode(state: {
    userCode: string
    verificationUri: string
    verificationUriComplete: string | null
  }) {
    if (dialogShown) return
    dialogShown = true
    const website = state.verificationUriComplete || state.verificationUri
    const caption = pclText(
      state.verificationUriComplete
        ? 'Launch.Account.LoginDialog.MicrosoftInstructions.WithAutoFill'
        : 'Launch.Account.LoginDialog.MicrosoftInstructions',
      [state.userCode, website],
    )
    // CE 在弹窗出现 2 秒后自动打开网页并复制授权码。
    setTimeout(() => {
      // 用户已经取消时不要再打开网页。
      if (account.loginState.status !== 'deviceCode') return
      void openExternal(website)
      void copyCode(state.userCode)
    }, AUTO_OPEN_DELAY)
    await PMsgLogin(state.userCode, website, caption, () => void account.cancelLogin())
    dialogShown = false
  }

  async function showProfileSelection(candidates: { id: string; name: string }[]) {
    const result = await modal.open({
      title: pclText('Minecraft.Launch.Login.Auth.SelectProfile'),
      width: ModalWidthVariant.Slim,
      selectOptions: candidates.map((candidate) => ({
        value: candidate.id,
        label: candidate.name,
      })),
      confirmText: pclText('Common.Action.Confirm'),
      cancelText: pclText('Common.Action.Cancel'),
    })
    await account.chooseCandidate(result.confirmed ? result.selected : null)
  }

  watch(
    () => account.loginState,
    async (state) => {
      switch (state.status) {
        case 'deviceCode':
          await showDeviceCode(state)
          return
        case 'selectProfile':
          await showProfileSelection(state.candidates)
          return
        case 'succeeded':
          modal.close(false)
          dialogShown = false
          sideTip.show(pclText('Launch.Account.LoginDialog.Success'), 'success')
          account.resetLoginState()
          onSucceeded?.()
          return
        case 'failed':
          modal.close(false)
          dialogShown = false
          account.resetLoginState()
          await reportLoginFailure(state, context())
          return
        case 'cancelled':
          modal.close(false)
          dialogShown = false
          account.resetLoginState()
          await reportLoginFailure({ code: CANCELLED, message: '' }, context())
          return
        default:
          return
      }
    },
  )
}
