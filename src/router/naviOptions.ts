export interface INavItem {
    itemName: string,
    icon?: any,
    linkto?: string
}

export interface INavItemGroup {
    title: string,
    content: INavItem[],
}