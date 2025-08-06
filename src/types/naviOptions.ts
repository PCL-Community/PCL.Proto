import type { VNodeTypes } from "vue";

export interface INavItem {
    text: string,
    icon?: VNodeTypes,
    linkto?: string
}

export interface INavItemGroup {
    title?: string,
    content: INavItem[],
}