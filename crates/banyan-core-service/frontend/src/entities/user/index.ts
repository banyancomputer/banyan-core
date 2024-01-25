export interface User {
    id: string;
    email: string;
    displayName: string;
    locale: string;
    profileImage: string;
    acceptedTosAt: number | null;
    accountTaxClass: string;
    subscriptionId: string;
};
