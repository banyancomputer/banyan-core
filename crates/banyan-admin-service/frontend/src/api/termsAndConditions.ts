import { RawUser } from '@app/types';
import { TermsAndConditions } from '@app/types/terms';
import { APIClient } from './http';

export class TermsAndColditionsClient extends APIClient {
    public async getTermsAndCondition(): Promise<TermsAndConditions> {
        const response = await this.http.get(`${this.ROOT_PATH}/tos`);

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    }

    public async confirmTermsAndConditions(userData: RawUser, accepted_tos_at: number): Promise<void> {
        const response = await this.http.put(`${this.ROOT_PATH}/api/v1/users/current`, JSON.stringify({ ...userData, accepted_tos_at }));

        if (!response.ok) {
            await this.handleError(response);
        }
    }
};
