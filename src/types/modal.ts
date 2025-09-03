import type { ButtonType } from "@/components/widget/PButton.vue";
import { type Ref } from "vue";

export interface ModalButtonOption {
  type?: ButtonType;
  content: string;
  operation?: () => void | Promise<void>;
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
}

export interface ModalApi {
  isOpen: Ref<boolean>,
  options: Ref<ModalOptions>,
  open: (options: ModalOptions) => Promise<boolean>,
  close: (result?: boolean) => void,
}

