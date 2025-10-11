import type { Component } from "vue";

export interface INavItem {
    text: string,
    icon?: Component,
    linkto?: string,
    clickCallback?: (payload: PointerEvent) => any,
}

export interface INavItemGroup {
    title?: string,
    content: INavItem[],
}