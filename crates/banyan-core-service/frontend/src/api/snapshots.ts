import { APIClient } from "./http";

export class SnapshotsClient extends APIClient {
    public async getSnapshots(bucketId: string){
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/buckets/${bucketId}/snapshots`);

        if (!response.ok) {
            await this.handleError(response);
        };

        return await response.json();
    };

    public async restoreFromSnapshot(bucketId: string, snapshotId: string){
        const response = await this.http.put(`${this.ROOT_PATH}/api/v1/buckets/${bucketId}/snapshots/${snapshotId}/restore`);

        if (!response.ok) {
            await this.handleError(response);
        };
    }
};