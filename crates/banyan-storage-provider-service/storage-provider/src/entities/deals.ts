export interface AvailiableDeal {
    id: string,
    size: string,
    payment: string,
    status: string,
    accept_by: string,
    sealed_by: string
};

export interface ActiveDeal {
    id: string,
    size: string,
    payment: string,
    status: string,
    accepted_at: string,
    canceled_at: string,
    sealed_by: string
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