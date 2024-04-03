export interface User {
    id: string;
    email: string;
    displayName: string;
    locale: string;
    monthlyEggress: number;
    profileImage: string;
    acceptedTosAt: number | null;
    accountTaxClass: string;
    subscriptionId: string;
    subscriptionValidUntil: string;
};
