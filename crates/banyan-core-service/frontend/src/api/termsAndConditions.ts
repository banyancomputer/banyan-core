import { APIClient } from "./http";
import { TermsAndConditions } from "@app/types/terms";

export class TermsAndColditionsClient extends APIClient {
    public async getTermsAndCondition(): Promise<TermsAndConditions> {
        /** TODO: implement when api will be implemented. */
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/tos`)

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    }
};
