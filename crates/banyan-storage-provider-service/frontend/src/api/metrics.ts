import { APIClient } from ".";

import { BandwidthUsage, OveralStatistic, StorageUsage } from "@/entities/metrics";

export class MetricsClient extends APIClient {

    public async getOverallStatistic(): Promise<OveralStatistic> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/metrics/current`)

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    };

    public async getBandwidthUsage(): Promise<BandwidthUsage[]> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/metrics/bandwidth/daily`)

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    };

    public async getStorageUsage(): Promise<StorageUsage[]> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/metrics/storage/daily`)

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    };
}