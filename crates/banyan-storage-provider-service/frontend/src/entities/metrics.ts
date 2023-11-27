export interface BandwidthUsage {
    date: [number, number]
    egress: number,
    ingress: number
};

export interface StorageUsage {
    date: [number, number]
    used: number,
    available: number
};

export interface OveralStatistic{
    bandwidth: {
        egress: number,
        ingress: number
    },
    deals: {
        accepted: number,
        sealed: number
    },
    storage: {
        available: number,
        used: number
    }
};
