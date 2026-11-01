import { pclText } from '@/api/pclLocale'
import IconDownload from '@/components/icons/header/IconDownload.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import { h, shallowReactive, type Component } from 'vue'

export enum FloatButtonType {
  DownloadGame,
  TaskManage,
  ModifyInstance,
  ExportInstance,
}

export interface FloatButtonState {
  visible: boolean
  text?: string
  icon?: Component
  title?: string
  type: FloatButtonType
  action?: () => void
}

const floatButtonState = shallowReactive<FloatButtonState>({
  visible: false,
  get text() {
    switch (this.type) {
      case FloatButtonType.DownloadGame:
        return pclText('Download.Install.StartDownload')
      case FloatButtonType.ModifyInstance:
        return pclText('Instance.Install.Action.StartModify')
      case FloatButtonType.ExportInstance:
        return pclText('Instance.Export.StartExport')
      default:
        return undefined
    }
  },
  get icon() {
    return this.type === FloatButtonType.ExportInstance
      ? () => h(PSvgIcon, { name: 'lucide/package', size: 16 })
      : IconDownload
  },
  get title() {
    return this.type === FloatButtonType.TaskManage ? pclText('Main.Title.TaskManager') : undefined
  },
  type: FloatButtonType.DownloadGame,
})

export const useFloatButton = () => {
  const set = (type: FloatButtonType) => {
    floatButtonState.type = type
  }
  return {
    floatButtonState,
    setFloatButton: set,
  }
}
