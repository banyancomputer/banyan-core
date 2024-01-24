export interface User {
    id: string;
    email: string;
    verifiedEmail: boolean;
    displayName: string;
    locale: string;
    profileImage: string;
    acceptedTosAt: number | null;
    subscriptionId: string;
};
