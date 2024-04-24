import { AcceptedDeal, ActiveDeal, DealState } from "@/entities/deals";
import { APIClient } from ".";

export class DealsClient extends APIClient {
    public async getDeals<T extends DealState>(state: T):
      Promise<T extends DealState.Accepted ? AcceptedDeal[] : ActiveDeal[]> {
        const url = `${this.ROOT_PATH}/api/v1/deals?state=${state}`;
        const response = await this.http.get(url);
        if (!response.ok) {
            throw await this.handleError(response);
        }
        return await response.json() as T extends DealState.Accepted ? AcceptedDeal[] : ActiveDeal[];
    };

    public async acceptDeal(id: string): Promise<void> {
        const response = await this.http.put(`${this.ROOT_PATH}/api/v1/deals/${id}/accept`)

        if (!response.ok) {
            throw await this.handleError(response);
        }
    };

    public async cancelDeal(id: string): Promise<void> {
        const response = await this.http.put(`${this.ROOT_PATH}/api/v1/deals/${id}/cancel`)

        if (!response.ok) {
            throw await this.handleError(response);
        }
    };

    public async downloadDeal(id: string): Promise<Blob> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/deals/${id}/download`, undefined, { 'Content-Type': 'application/vnd.ipld.car'})

        if (!response.ok) {
            throw await this.handleError(response);
        }

        return await response.blob();
    };

    public async sealDeal(id: string): Promise<void> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/deals/${id}/seal`)

        if (!response.ok) {
            throw await this.handleError(response);
        }
    };
}