import { type Ref } from "vue";

export interface ModalOptions {
  content?: string;
  title?: string;
}

export interface ModalApi {
  isOpen: Ref<boolean>,
  options: Ref<ModalOptions>,
  open: (options: ModalOptions) => Promise<boolean>,
  close: (result?: boolean) => void,
}

