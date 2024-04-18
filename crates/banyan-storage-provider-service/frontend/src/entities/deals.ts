export interface AvailableDeal {
    id: string,
    size: number,
    state: string,
    accepted_by: string | null,
    accepted_at: string | null
};

export interface ActiveDeal {
    id: string,
    size: number,
    state: string,
    accepted_by: string | null,
    accepted_at: string | null
};

export interface Metrics {
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