import { APIClient } from './http';
import { User } from '@/entities/user';

export class UserClient extends APIClient {
    public async getCurrentUser(): Promise<User> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/users/current`);

        if (!response.ok) {
            await this.handleError(response);
        }

        const rawUser = await response.json();

        return {
            id: rawUser.id,
            email: rawUser.email,
            displayName: rawUser.display_name,
            locale: rawUser.locale,
            profileImage: rawUser.profile_image,
            acceptedTosAt: rawUser.accepted_tos_at,
			accountTaxClass: rawUser.account_tax_class,
            subscriptionId: rawUser.subscription_id,
            subscriptionValidUntil: rawUser.subscription_valid_until,
        };
    };
};
