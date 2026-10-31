export type ExtraItem =
  string | { text: string; extra?: ExtraItem[]; bold?: boolean; color?: string }

export interface MCPingResult {
  version: {
    name: string
    protocol: number
  }
  players: {
    max: number
    online: number
    sample?: {
      name: string
      id: string
    }[]
  }
  description: ExtraItem
  favicon?: string
  modinfo?: {
    type: string
    modList: {
      id: string
      version: string
    }[]
  }
}
