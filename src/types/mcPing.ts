interface IPEndPoint {
    address: string
    port: number
}

export interface McPingResult {
    version: {
        name: string,
        protocol: number
    }
    players: {
        max: number,
        online: number,
        samples: {
            name: string,
            id: string
        }[]
    },
    description: string,
    favicon: string,
    latency: number,
    modInfo: {
        type: string,
        modList: {
            id: string,
            version: string
        }[]
    }
}