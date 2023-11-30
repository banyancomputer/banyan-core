import { APIClient } from "./http";

export class UserClient extends APIClient {
    public async getCurrentUser() {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/users/current`);

        if (!response.ok) {
            await this.handleError(response);
        }
    }
};
