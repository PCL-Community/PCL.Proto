import { pclText } from '@/api/pclLocale'
import { useModal } from '@/composables/useModal'

export function showUnavailable() {
  return useModal().open({
    title: pclText('Common.Dialog.Title'),
    content: pclText('Proto.NotImplemented.Action'),
    buttons: [{ type: 'tint', content: pclText('Common.Action.Confirm') }],
  })
}
