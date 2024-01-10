import { StorageProvider, Deal } from '@app/types';
import { APIClient } from './http';


export class AdminClient extends APIClient {
    public async getDeals(): Promise<StorageProvider[]> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/deals`);

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    }

    public async getStorageProviders(): Promise<StorageProvider[]> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/providers`);

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    }


    public async getStorageProviderById(id: string): Promise<StorageProvider> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/providers/${id}`);

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    }

    public async getDealById(id: string): Promise<Deal> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/deals/${id}`);

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    }


};
