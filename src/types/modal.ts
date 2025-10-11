import type { ButtonType } from "@/components/widget/PButton.vue";
import { type Ref } from "vue";

export interface ModalButtonOption {
  type?: ButtonType;
  content: string;
  operation?: () => any | Promise<any>;
}

export interface ModalResult {
  confirmed: boolean
  input?: string
}

export enum ModalWidthVirant {
  Slim = 400,
  Fat = 560
}

export interface ModalOptions {
  content?: string;
  title?: string;
  width?: ModalWidthVirant;
  buttons?: ModalButtonOption[];
  showInput?: boolean,
  defaultInputText?: string;
}

export interface ModalApi {
  isOpen: Ref<boolean>,
  options: Ref<ModalOptions>,
  inputValue: Ref<string | undefined>,
  open: (options: ModalOptions) => Promise<ModalResult>,
  close: (result?: boolean) => void,
}

