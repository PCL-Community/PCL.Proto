import { computed, ref, type Ref } from 'vue'
import { type ModalApi, type ModalOptions } from '@/types/modal'

const state = ref<{ isOpen: boolean, options: ModalOptions }>({ isOpen: false, options: {} })

const resolvePromise: Ref<((value: boolean) => void) | null> = ref(null)

export function useModal(): ModalApi {
  const open = (options: ModalOptions): Promise<boolean> => {
    state.value.options.content = options.content || '模态框内容'
    state.value.options.title = options.title || '模态框标题'
    state.value.isOpen = true

    return new Promise((resolve) => {
      resolvePromise.value = resolve
    })
  }

  const close = (result: boolean = false): void => {
    state.value.isOpen = false
    if (resolvePromise.value) {
      resolvePromise.value(result)
      resolvePromise.value = null
    }
  }

  return {
    isOpen: computed(() => state.value.isOpen),
    options: computed(() => state.value.options),
    open,
    close,
  }
}
