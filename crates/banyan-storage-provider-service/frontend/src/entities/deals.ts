export interface ActiveDeal {
    id: string,
    size: number,
    state: string,
    payment: number,
    status: DealState,
    accept_by: string,
    requested_at: string,
};

export interface AcceptedDeal {
    id: string,
    payment: string,
    size: number,
    state: DealState,
    accept_by: string | null,
    accepted_at: string | null,
    seal_by: string | null
};

export enum DealState {
    Accepted = 'accepted',
    Active = 'active',
}

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
        active: number,
        used: number
    }
};