export type SideTipType = 'default' | 'warn' | 'success'

export interface ISideTipItem {
  id: number
  message: any
  colorType: SideTipType
}
