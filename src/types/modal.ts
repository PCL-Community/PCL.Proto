import type { ButtonType } from "@/components/widget/MyButton.vue";
import { type Ref } from "vue";

export interface MyButtonOption {
  type: ButtonType;
  content: string;
  operation: any;
}

export enum ModalWidthVirant {
  Slim = 400,
  Fat = 560
}

export interface ModalOptions {
  content?: string;
  title?: string;
  width?: ModalWidthVirant;
}

export interface ModalApi {
  isOpen: Ref<boolean>,
  options: Ref<ModalOptions>,
  open: (options: ModalOptions) => Promise<boolean>,
  close: (result?: boolean) => void,
}

