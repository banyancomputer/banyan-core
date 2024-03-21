import { APIClient } from './http';

export class AccessKeysClient extends APIClient {
    public async deleteAccessKey(bucketId: string, keyId: string) {
        const response = await this.http.delete(`${this.ROOT_PATH}/api/v1/${bucketId}/keys/${keyId}`);

        if (!response.ok) {
            await this.handleError(response);
        }
    };
}
