import { pclText } from '@/api/pclLocale'
import sideTip from '@/composables/sideTip'
import { useModal } from '@/composables/useModal'
import { ModalWidthVariant, type ModalButtonOption } from '@/types/modal'
import { openExternal } from '@/util/externalLink'
import { CANCELLED, type AuthFailure } from '@/stores/account'

/** CE 在各处错误弹窗里打开的网址。 */
const URLS = {
  purchase:
    'https://www.xbox.com/zh-cn/games/store/minecraft-java-bedrock-edition-for-pc/9nxp44l49shj',
  createProfile: 'https://www.minecraft.net/zh-hans/msaprofile/mygames/editprofile',
  xboxSignup: 'https://signup.live.com/signup',
  birthDate: 'https://account.live.com/editprof.aspx',
  birthDateSupport:
    'https://support.microsoft.com/zh-cn/account-billing/如何更改-microsoft-帐户上的出生日期-837badbc-999e-54d2-2617-d19206b9540a',
  password: 'https://account.live.com/password/Change',
}

export { URLS as loginUrls }

interface DialogOptions {
  title: string
  content: string
  ok: string
  cancel?: string
  /** 第三个按钮，只有 CE 的未成年账号弹窗用到。 */
  extra?: string
}

interface DialogChoice {
  confirmed: boolean
  /** 1 = 第一个按钮，2 = 第二个，3 = 第三个（对照 CE 的 MyMsgBox 返回值）。 */
  choice: number
}

/** 打开一个按钮弹窗，返回用户点了哪个按钮。 */
export async function dialog(options: DialogOptions): Promise<DialogChoice> {
  const modal = useModal()
  let choice = 0
  const press = (value: number, confirmed: boolean) => () => {
    choice = value
    modal.close(confirmed)
  }
  const buttons: ModalButtonOption[] = [
    { content: options.ok, type: 'tint', operation: press(1, true) },
  ]
  if (options.extra) buttons.push({ content: options.extra, operation: press(3, false) })
  if (options.cancel) buttons.push({ content: options.cancel, operation: press(2, false) })
  await modal.open({
    title: options.title,
    content: options.content,
    width: ModalWidthVariant.Slim,
    buttons,
  })
  return { confirmed: choice === 1, choice }
}

/**
 * 把登录失败映射成 CE 的对应提示，返回是否已经处理成用户可理解的提示。
 *
 * 对照 `ProfileUi.HandleMicrosoftXstsError` / `HandleMicrosoftNotOwnedError` /
 * `HandleMicrosoftCreateProfileError`，以及两个登录页对普通错误的处理。
 */
export async function reportLoginFailure(failure: AuthFailure, context: 'microsoft' | 'auth') {
  switch (failure.code) {
    case CANCELLED:
      sideTip.show(
        pclText(
          context === 'auth' ? 'Launch.Account.Auth.Cancelled' : 'Launch.Account.LoginCancelled',
        ),
      )
      return
    case 'xsts_2148916227':
      await dialog({
        title: pclText('Minecraft.Launch.Login.Failed'),
        content: pclText('Minecraft.Launch.Login.Microsoft.Banned'),
        ok: pclText('Minecraft.Launch.Login.IKnow'),
      })
      return
    case 'xsts_2148916233': {
      const { confirmed } = await dialog({
        title: pclText('Minecraft.Launch.Login.Hint'),
        content: pclText('Minecraft.Launch.Login.Microsoft.XboxNotRegistered'),
        ok: pclText('Minecraft.Launch.Login.Register'),
        cancel: pclText('Common.Action.Cancel'),
      })
      if (confirmed) await openExternal(URLS.xboxSignup)
      return
    }
    case 'xsts_2148916235':
      await dialog({
        title: pclText('Minecraft.Launch.Login.Failed'),
        content: pclText('Minecraft.Launch.Login.Microsoft.RegionBlocked'),
        ok: pclText('Minecraft.Launch.Login.IKnow'),
      })
      return
    case 'xsts_2148916238': {
      // CE：先问账号填写的年龄是否满 13 岁，再给出对应的修改入口。
      const { choice } = await dialog({
        title: pclText('Minecraft.Launch.Login.Hint'),
        content: pclText('Minecraft.Launch.Login.Microsoft.Underage.Message'),
        ok: pclText('Minecraft.Launch.Login.Microsoft.Underage.AgeOver13'),
        extra: pclText('Common.Option.IDontKnow'),
        cancel: pclText('Minecraft.Launch.Login.Microsoft.Underage.AgeUnder13'),
      })
      if (choice === 1) {
        await openExternal(URLS.birthDate)
        await dialog({
          title: pclText('Minecraft.Launch.Login.Hint'),
          content: pclText('Minecraft.Launch.Login.Microsoft.ChangeBirthDate.Message'),
          ok: pclText('Minecraft.Launch.Login.IKnow'),
        })
      } else if (choice !== 3) {
        await openExternal(URLS.birthDateSupport)
        await dialog({
          title: pclText('Minecraft.Launch.Login.Hint'),
          content: pclText('Minecraft.Launch.Login.Microsoft.ChangeBirthDate.SupportMessage'),
          ok: pclText('Minecraft.Launch.Login.IKnow'),
        })
      }
      return
    }
    case 'minecraft_not_owned': {
      const { confirmed } = await dialog({
        title: pclText('Minecraft.Launch.Login.Failed'),
        content: pclText('Minecraft.Launch.Login.Microsoft.NotPurchased'),
        ok: pclText('Minecraft.Launch.Login.Microsoft.PurchaseMinecraft'),
        cancel: pclText('Common.Action.Cancel'),
      })
      if (confirmed) await openExternal(URLS.purchase)
      return
    }
    case 'profile_not_created': {
      const { confirmed } = await dialog({
        title: pclText('Minecraft.Launch.Login.Failed'),
        content: pclText('Minecraft.Launch.Login.Microsoft.CreateProfile.Message'),
        ok: pclText('Minecraft.Launch.Login.Microsoft.CreateProfile.Button'),
        cancel: pclText('Common.Action.Cancel'),
      })
      if (confirmed) await openExternal(URLS.createProfile)
      return
    }
    case 'network_error':
      // CE 对网络类错误给出检查网络的提示，并保留原始信息。
      await dialog({
        title: pclText(
          context === 'auth'
            ? 'Minecraft.Launch.Login.Auth.FailedTitle'
            : 'Launch.Account.Microsoft.LoginFailed.Title',
        ),
        content: pclText(
          context === 'auth'
            ? 'Minecraft.Launch.Login.Auth.NetworkFailed.WithDetail'
            : 'Launch.Account.Microsoft.LoginFailed.WithDetail',
          [failure.message],
        ),
        ok: pclText('Minecraft.Launch.Login.IKnow'),
      })
      return
    default:
      // 认证服务器给出的原因直接提示，其余情况弹窗展示详情（CE 的两种分支）。
      sideTip.show(
        pclText(
          context === 'auth'
            ? 'Launch.Account.Auth.LoginFailed.WithDetail'
            : 'Launch.Account.Microsoft.LoginFailed.WithDetail',
          [failure.message || failure.code],
        ),
        'warn',
      )
      return
  }
}
