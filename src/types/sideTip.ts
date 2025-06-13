export type SideTipType = 'default' | 'warn'

export interface ISideTipItem {
    id: number
    message: string
    colorType: SideTipType
}