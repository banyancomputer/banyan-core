export interface Invoice {
    id: string;
    billing_start: string;
    billing_end: string;
    subscription_id: string,
    total_amount: number;
    status:  string;
    created_at: string;
};

export interface Subscription {
    currently_active: boolean;
    features: {
        archival_available: boolean;
        archival_hard_limit?: number;
        hot_storage_hard_limit?: number;
        included_bandwidth: number;
        included_hot_replica_count: number;
        included_hot_storage: number;

    },
    id: string;
    service_key: string;
    tax_class: string;
    title: string;
    pricing?: {
        archival: number;
        bandwidth: number;
        hot_storage: number;
        plan_base: number;
    }
}
