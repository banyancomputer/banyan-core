import { Deal, StorageHost, StorageHostRequest, User } from '@app/types';
import { APIClient } from './http';

export class AdminClient extends APIClient {

	public async resetUser(userId: string): Promise<void> {
		const response = await this.http.delete(`${this.ROOT_PATH}/api/v1/admin/users/${userId}`);

		if (!response.ok) {
			await this.handleError(response);
		}
	}
	public async getCurrentUser(): Promise<User> {
		const response = await this.http.get(`${this.ROOT_PATH}/api/v1/users/current`);

		if (!response.ok) {
			await this.handleError(response);
		}

		const rawUser = await response.json();

		return {
			id: rawUser.id,
			email: rawUser.email,
			verifiedEmail: rawUser.verified_email,
			displayName: rawUser.display_name,
			locale: rawUser.locale,
			profileImage: rawUser.profile_image,
			acceptedTosAt: rawUser.accepted_tos_at,
		};
	};

	public async getDeals(): Promise<Deal[]> {
		const response = await this.http.get(
			`${this.ROOT_PATH}/api/v1/admin/deals`
		);

		if (!response.ok) {
			await this.handleError(response);
		}

		return await response.json();
	}

	public async getUsers(): Promise<User[]> {
		const response = await this.http.get(
			`${this.ROOT_PATH}/api/v1/admin/users`
		);

		if (!response.ok) {
			await this.handleError(response);
		}

		return await response.json();
	}

	public async resetUserAccount(id: string): Promise<User> {
		const response = await this.http.post(
			`${this.ROOT_PATH}/api/v1/admin/users/${id}/reset`
		);

		if (!response.ok) {
			await this.handleError(response);
		}

		return await response.json();
	}

	public async getStorageHosts(): Promise<StorageHost[]> {
		const response = await this.http.get(
			`${this.ROOT_PATH}/api/v1/admin/providers`
		);

		if (!response.ok) {
			await this.handleError(response);
		}

		return await response.json();
	}

	public async createStorageHost(
		createHost: StorageHostRequest
	): Promise<StorageHost> {
		const response = await this.http.post(
			`${this.ROOT_PATH}/api/v1/admin/providers`,
			JSON.stringify(createHost)
		);

		if (!response.ok) {
			await this.handleError(response);
		}

		return await response.json();
	}

	public async getStorageHostById(id: string): Promise<StorageHost> {
		const response = await this.http.get(
			`${this.ROOT_PATH}/api/v1/admin/providers/${id}`
		);

		if (!response.ok) {
			await this.handleError(response);
		}

		return await response.json();
	}

	public async getDealById(id: string): Promise<Deal> {
		const response = await this.http.get(
			`${this.ROOT_PATH}/api/v1/admin/deals/${id}`
		);

		if (!response.ok) {
			await this.handleError(response);
		}

		return await response.json();
	}
}
