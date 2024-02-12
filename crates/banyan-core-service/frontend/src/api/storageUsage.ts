import { APIClient } from "./http";

export class StorageUsageClient extends APIClient {
    async getStorageUsage ():Promise<{size: number}> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/buckets/usage`);

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    };

    async getStorageLimits ():Promise<{soft_hot_storage_limit: number, hard_hot_storage_limit: number, size: number}> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/buckets/usage_limit`);

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    };
};
