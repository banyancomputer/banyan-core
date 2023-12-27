/**
 * id: the user's id
 * email: the user's email
 * verifiedEmail: whether or not their email is verified
 * displayName: display name (if any)
 * locale: user locale
 * profileImage: link to the user's profile image
 */
export interface User {
    id: string;
    email: string;
    verifiedEmail: boolean;
    displayName: string;
    locale: string;
    profileImage: string;
    acceptedTosAt: number | null;
};

export interface RawUser {
    id: string;
    email: string;
    verified_email: boolean;
    display_name: string;
    locale: string;
    profile_image: string;
    accepted_tos_at: number | null;
};
