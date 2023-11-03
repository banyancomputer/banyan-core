import { EscrowedKeyMaterial } from "@app/lib/crypto/types";

/**
 * Represents an OAuth2 account from NextAuth.js
 * Do not change!
 */
export interface Account {
    id: string;
    userId: string;
    type: string;
    provider: string;
    providerAccountId: string;
    refresh_token: string | null;
    access_token: string | null;
    expires_at: number | null;
    token_type: string | null;
    scope: string | null;
    id_token: string | null;
    session_state: string | null;
};

export interface SessionData {
    accountId: string;
    email: string,
    verified_email: boolean,
    name: string,
    locale: string,
    image: string,
    escrowedKey: EscrowedKeyMaterial
}
