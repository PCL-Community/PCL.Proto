export type ExtraItem =
  | string
  | { text: string; extra?: ExtraItem[]; bold?: boolean; color?: string }

export interface MCPingResult {
  version: {
    name: string
    protocol: number
  }
  players: {
    max: number
    online: number
    samples?: {
      name: string
      id: string
    }[]
  }
  description: ExtraItem
  favicon?: string
  modInfo?: {
    type: string
    modList: {
      id: string
      version: string
    }[]
  }
}
