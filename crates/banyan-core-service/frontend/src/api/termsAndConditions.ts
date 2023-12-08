import { RawUser } from "@app/types";
import { APIClient } from "./http";
import { TermsAndConditions } from "@app/types/terms";

export class TermsAndColditionsClient extends APIClient {
    public async getTermsAndCondition(): Promise<TermsAndConditions> {
        /** TODO: implement when api will be implemented. */
        const response = await this.http.get(`${this.ROOT_PATH}/tos`)

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    }

    public async confirmTermsAndConditions(userData: RawUser, accepted_tos_at: number): Promise<void> {
        const response = await this.http.put(`${this.ROOT_PATH}/api/v1/users/current`, JSON.stringify({...userData, accepted_tos_at}));

        if (!response.ok) {
            await this.handleError(response);
        }
    }
};
