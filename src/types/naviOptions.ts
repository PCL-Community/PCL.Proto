export interface INavItem {
    text: string,
    icon?: any,
    linkto?: string
}

export interface INavItemGroup {
    title?: string,
    content: INavItem[],
}