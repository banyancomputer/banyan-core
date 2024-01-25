import { APIClient } from "./http";
import { Subscription } from "@/entities/billing";

export class BillingClient extends APIClient {
    public async getInvoices(): Promise<any> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/invoices`);

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    };

    public async getInvoiceById(id: string): Promise<any> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/invoices/${id}`);

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    };

    public async getSubscriptions(): Promise<Subscription[]> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/subscriptions`);

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    };

    public async getSubscriptionById(id: string): Promise<Subscription> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/subscriptions/${id}`);

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    };

    public async subscribe(id: string): Promise<{checkout_url: string}> {
        const response = await this.http.post(`${this.ROOT_PATH}/api/v1/subscriptions/${id}/subscribe`);

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    };

    public async manage(): Promise<{portal_url: string}> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/subscriptions/manage`);

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    };
}