export type SideTipType = 'default' | 'warn' | 'success'

export interface ISideTipItem {
    id: number
    message: string
    colorType: SideTipType
}