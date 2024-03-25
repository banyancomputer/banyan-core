import { StorageLimits, StorageUsage } from "@/entities/storage";
import { APIClient } from "./http";

export class StorageUsageClient extends APIClient {
    async getStorageUsage():Promise<StorageUsage> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/buckets/usage`);

        if (!response.ok) {
            await this.handleError(response);
        }

        const rawStorageUsage = await response.json();

        return new StorageUsage(rawStorageUsage.hot_storage, rawStorageUsage.archival_storage);
    };

    async getStorageLimits():Promise<StorageLimits> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/buckets/usage_limit`);

        if (!response.ok) {
            await this.handleError(response);
        }

        const rawStorageLimits = await response.json();

        return new StorageLimits(rawStorageLimits.soft_hot_storage_limit, rawStorageLimits.hard_hot_storage_limit, rawStorageLimits.size);
    };
};
