import { EscrowedKeyMaterial, User } from '@/app/types';
import { parseCookies, setCookie, destroyCookie } from 'nookies';

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

	const cookies = parseCookies();
	return cookies[SESSION_KEY_COOKIE_NAME];
}

export const getLocalKey = (): LocalKey => {
	const cookies = parseCookies();
	if (!cookies[LOCAL_KEY_COOKIE_NAME]) {
		const id = Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
		const key = Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
		setCookie(null, LOCAL_KEY_COOKIE_NAME, `${id}:${key}`, {
			maxAge: COOKIE_MAX_AGE,
			// lax: process.env.NODE_ENV === 'development',
			sameSite: 'strict',
			path: '/',
		});
		return { id, key };
	}
	const [id, key] = cookies[LOCAL_KEY_COOKIE_NAME].split(':');
	return { id, key };
}

export const destroyLocalKey = () => {
	destroyCookie(null, LOCAL_KEY_COOKIE_NAME)
}

export const getUserData = (): UserData | null => {
	const cookies = parseCookies();
	if (!cookies[USER_DATA_COOKIE_NAME]) {
		return null;
	}
	const userDataJson = JSON.parse(cookies[USER_DATA_COOKIE_NAME]);
	let escrowedKeyMaterial = null;
	if (userDataJson.escrowed_key_material) {
		escrowedKeyMaterial = {
			apiPublicKeyPem: userDataJson.escrowed_key_material.api_public_key_pem,
			encryptionPublicKeyPem: userDataJson.escrowed_key_material.encryption_public_key_pem,
			encryptedPrivateKeyMaterial: userDataJson.escrowed_key_material.encrypted_private_key_material,
			passKeySalt: userDataJson.escrowed_key_material.pass_key_salt
		} as EscrowedKeyMaterial;
	}
	let user = {
		id: userDataJson.user.id,
		email: userDataJson.user.email,
		verifiedEmail: userDataJson.user.verified_email,
		displayName: userDataJson.user.display_name,
		locale: userDataJson.user.locale,
		profileImage: userDataJson.user.profile_image,
		acceptedTosAt: userDataJson.user.accepted_tos_at
	} as User;

	return {
		user, escrowedKeyMaterial
	}
}

export const setUserDataEscrowedKeyMaterial = (escrowedKeyMaterial: EscrowedKeyMaterial) => {
	const cookies = parseCookies();
	if (!cookies[USER_DATA_COOKIE_NAME]) {
		return;
	}
	const userDataJson = JSON.parse(cookies[USER_DATA_COOKIE_NAME]);
	userDataJson.escrowed_key_material = {
		api_public_key_pem: escrowedKeyMaterial.apiPublicKeyPem,
		encryption_public_key_pem: escrowedKeyMaterial.encryptionPublicKeyPem,
		encrypted_private_key_material: escrowedKeyMaterial.encryptedPrivateKeyMaterial,
		pass_key_salt: escrowedKeyMaterial.passKeySalt
	};
	setCookie(null, USER_DATA_COOKIE_NAME, JSON.stringify(userDataJson), {
		maxAge: COOKIE_MAX_AGE,
		// lax: process.env.NODE_ENV === 'development',
		sameSite: 'strict',
		path: '/',
	});
}