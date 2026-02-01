import IconDownload from '@/components/icons/header/IconDownload.vue'
import { shallowReactive, type Component } from 'vue'

export enum FloatButtonType {
  DownloadGame,
  TaskManage,
}

export interface FloatButtonState {
  visible: boolean
  text?: string
  icon?: Component
  title?: string
  type: FloatButtonType
}

const floatButtonState = shallowReactive<FloatButtonState>({
  visible: false,
  text: '开始下载',
  icon: IconDownload,
  title: undefined as string | undefined,
  type: FloatButtonType.DownloadGame,
})

export const useFloatButton = () => {
  const set = (type: FloatButtonType) => {
    switch (type) {
      case FloatButtonType.DownloadGame:
        floatButtonState.text = '开始下载'
        break
      case FloatButtonType.TaskManage:
        floatButtonState.text = undefined
        floatButtonState.title = '任务管理'
        break
    }
    floatButtonState.type = type
  }
  return {
    floatButtonState,
    setFloatButton: set,
  }
}
