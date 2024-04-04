import { APIClient } from './http';

export class AccessKeysClient extends APIClient {
    public async deleteAccessKey(bucketId: string, userKeyId: string) {
        const response = await this.http.delete(`${this.ROOT_PATH}/api/v1/${bucketId}/access/${userKeyId}`);

        if (!response.ok) {
            await this.handleError(response);
        }
    };
}
