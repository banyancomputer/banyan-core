import { ActiveDeal, AvailiableDeal } from "@/entities/deals";
import { APIClient } from ".";

export class DealsClient extends APIClient {
    public async getActiceDeals():Promise<ActiveDeal[]> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/deals`)

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    };
    public async getAvailableDeals():Promise<AvailiableDeal[]> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/deals/available`)

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    };

    public async acceptDeal(id: string): Promise<void> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/deals/${id}/accept`)

        if (!response.ok) {
            await this.handleError(response);
        }
    };

    public async declineDeal(id: string): Promise<void> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/deals/${id}/cancel`)

        if (!response.ok) {
            await this.handleError(response);
        }
    };

    public async downloadDeal(id: string): Promise<any> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/deals/${id}/download`)

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json()
    };

    public async proofDeal(id: string): Promise<void> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/deals/${id}/proof`)

        if (!response.ok) {
            await this.handleError(response);
        }
    };
}