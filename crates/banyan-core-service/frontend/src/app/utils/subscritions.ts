import { Subscription } from "@/entities/billing";


export const getHotStorageAmount = (subscription: Subscription | null) => {
    if (!subscription?.features?.included_hot_storage || !subscription?.features?.included_hot_replica_count) {
        return 0;
    };

    return subscription!.features.included_hot_storage / subscription!.features.included_hot_replica_count;
};
