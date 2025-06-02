import { ref } from 'vue'

const isOpen = ref(false)
const content = ref('')
const title = ref('')
const resolvePromise = ref(null)

export function useModal() {
  const open = (options = {}) => {
    content.value = options.content || ''
    title.value = options.title || ''
    isOpen.value = true

    return new Promise((resolve) => {
      resolvePromise.value = resolve
    })
  }

  const close = (result) => {
    isOpen.value = false
    if (resolvePromise.value) {
      resolvePromise.value(result)
      resolvePromise.value = null
    }
  }

  return {
    isOpen,
    content,
    title,
    open,
    close,
  }
}
