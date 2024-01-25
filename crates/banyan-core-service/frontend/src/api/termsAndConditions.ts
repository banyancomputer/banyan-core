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
    }

    public async confirmTermsAndConditions(userData: User, accepted_tos_at: number): Promise<void> {
        const rawUser = {
            id: userData.id,
            email: userData.email,
            verified_email: userData.verifiedEmail,
            display_name: userData.displayName,
            locale: userData.locale,
            profile_image: userData.profileImage,
            accepted_tos_at,
            subscription_id: userData.subscriptionId
        };

        const response = await this.http.put(`${this.ROOT_PATH}/api/v1/users/current`, JSON.stringify(rawUser));

        if (!response.ok) {
            await this.handleError(response);
        }
    }
};


{}