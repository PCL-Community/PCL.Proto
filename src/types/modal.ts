import type { ButtonType } from '@/components/widget/PButton.vue'
import { type Ref } from 'vue'

export interface ModalButtonOption {
  type?: ButtonType
  content: string
  operation?: () => any | Promise<any>
}

export interface ModalSelectOption {
  value: string
  label: string
  icon?: string
}

export interface ModalResult {
  confirmed: boolean
  input?: string
  selected?: string
}

export enum ModalWidthVariant {
  Slim = 400,
  Fat = 560,
}

export interface ModalOptions {
  content?: string
  title?: string
  width?: ModalWidthVariant
  buttons?: ModalButtonOption[]
  showInput?: boolean
  defaultInputText?: string
  selectOptions?: ModalSelectOption[]
  confirmText?: string
  cancelText?: string
}

export interface ModalApi {
  isOpen: Ref<boolean>
  options: Ref<ModalOptions>
  inputValue: Ref<string | undefined>
  selectedValue: Ref<string | undefined>
  open: (options: ModalOptions) => Promise<ModalResult>
  close: (result?: boolean) => void
}
