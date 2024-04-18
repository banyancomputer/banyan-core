import { ActiveDeal, AvailableDeal } from "@/entities/deals";
import { APIClient } from ".";

export class DealsClient extends APIClient {
    public async getDeals(status?: string):Promise<ActiveDeal[]> {
        const url = status ? `${this.ROOT_PATH}/api/v1/deals?status=${status}` : `${this.ROOT_PATH}/api/v1/deals`;
        const response = await this.http.get(url);
        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    };
    public async acceptDeal(id: string): Promise<void> {
        const response = await this.http.put(`${this.ROOT_PATH}/api/v1/deals/${id}/accept`)

        if (!response.ok) {
            await this.handleError(response);
        }
    };

    public async rejectDeal(id: string): Promise<void> {
        const response = await this.http.put(`${this.ROOT_PATH}/api/v1/deals/${id}/reject`)

        if (!response.ok) {
            await this.handleError(response);
        }
    };

    public async downloadDeal(id: string): Promise<Blob> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/deals/${id}/download`, undefined, { 'Content-Type': 'application/vnd.ipld.car'})

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.blob();
    };

    public async proofDeal(id: string): Promise<void> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/deals/${id}/proof`)

        if (!response.ok) {
            await this.handleError(response);
        }
    };
}