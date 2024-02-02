import { TermsAndConditions } from '@app/types/terms';
import { APIClient } from './http';
import { User } from '@/entities/user';

export class TermsAndColditionsClient extends APIClient {
    public async getTermsAndCondition(): Promise<TermsAndConditions> {
        const response = await this.http.get(`${this.ROOT_PATH}/tos`);

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    };

    public async confirmTermsAndConditions(userData: User, accepted_tos_at: number, account_tax_class: string): Promise<void> {
        const rawUser = {
            id: userData.id,
            accepted_tos_at,
			account_tax_class,
        };

        const response = await this.http.patch(`${this.ROOT_PATH}/api/v1/users/current`, JSON.stringify(rawUser));

        if (!response.ok) {
            await this.handleError(response);
        }
    };
};
