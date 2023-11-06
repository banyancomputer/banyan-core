import { EscrowedKeyMaterial, User } from '@/app/types';
import * as Cookies from 'js-cookie';
import Nookies from 'nookies';

/* Cookie State Management. This should probably be within a context but watching for cookie changes proved difficult */

const COOKIE_MAX_AGE = 60 * 60 * 24 * 7 * 4 * 3; // 3 months

// Cookie names
const SESSION_KEY_COOKIE_NAME = '_session_id';
const USER_DATA_COOKIE_NAME = '_user_data';
const LOCAL_KEY_COOKIE_NAME = '_local_key';

export interface LocalKey {
	id: string,
	key: string
}

export interface UserData {
	user: User,
	escrowedKeyMaterial: EscrowedKeyMaterial | null
}

export const getSessionKey = (): string | null => {
	const cookies = Nookies.get();
	return cookies[SESSION_KEY_COOKIE_NAME];
}

export const getLocalKey = (): LocalKey => {
	const cookies = Nookies.get();

	if (!cookies[LOCAL_KEY_COOKIE_NAME]) {
		const id = Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
		const key = Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
		Nookies.set(null,LOCAL_KEY_COOKIE_NAME, `${id}:${key}`, {
			maxAge: COOKIE_MAX_AGE,
			lax: process.env.NODE_ENV === 'development',
			sameSite: 'strict',
			path: '/',
		});
		return { id, key };
	}
	const [id, key] = cookies[LOCAL_KEY_COOKIE_NAME].split(':');
	return { id, key };
}

export const destroyLocalKey = () => {
	Cookies.remove(LOCAL_KEY_COOKIE_NAME)
}

export const getUserData = (): UserData | null => {
	console.log("getUserData");
	const cookies = Nookies.get();
	console.log(cookies);

	if (!cookies[USER_DATA_COOKIE_NAME]) {
		console.log("huh");
		return null;
	}
	console.log("huh");
	const userDataRaw = cookies[USER_DATA_COOKIE_NAME];
	console.log("raw: ", userDataRaw);
	const userDataJson = JSON.parse(userDataRaw);
	let escrowedKeyMaterial = null;
	if (userDataJson.escrowed_key_material) {
		escrowedKeyMaterial = {
			apiPublicKeyPem: userDataJson.api_public_key_pem,
			encryptionPublicKeyPem: userDataJson.encryption_public_key_pem,
			encryptedPrivateKeyMaterial: userDataJson.encrypted_private_key_material,
			passKeySalt: userDataJson.pass_key_salt
		} as EscrowedKeyMaterial;
	}
	let user = {
		id: userDataJson.id,
		email: userDataJson.email,
		verifiedEmail: userDataJson.verified_email,
		displayName: userDataJson.display_name,
		locale: userDataJson.locale,
		profileImage: userDataJson.profile_image
	}

	return {
		user, escrowedKeyMaterial
	}
}
