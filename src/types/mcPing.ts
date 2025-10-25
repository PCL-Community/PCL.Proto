export interface MCPingResult {
    version: {
        name: string,
        protocol: number
    }
    players: {
        max: number,
        online: number,
        samples?: {
            name: string,
            id: string
        }[]
    },
    description: string,
    favicon?: string,
    // modInfo: {
    //     type: string,
    //     modList: {
    //         id: string,
    //         version: string
    //     }[]
    // }
}