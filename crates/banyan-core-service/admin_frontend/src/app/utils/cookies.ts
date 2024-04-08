import { User } from '@/app/types';
import { parseCookies, setCookie, destroyCookie } from 'nookies';

/* Cookie State Management. This should probably be within a context but watching for cookie changes proved difficult */

const COOKIE_MAX_AGE = 60 * 60 * 24 * 7 * 4 * 3; // 3 months

// Cookie names
const SESSION_KEY_COOKIE_NAME = '_session_id';
const LOCAL_KEY_COOKIE_NAME = '_local_key';

export interface LocalKey {
	id: string;
	key: string;
}

export interface UserData {
	user: User;
}

export const getSessionKey = (): string | null => {
	const cookies = parseCookies();
	return cookies[SESSION_KEY_COOKIE_NAME];
};

export const getLocalKey = (): LocalKey => {
	const cookies = parseCookies();
	if (!cookies[LOCAL_KEY_COOKIE_NAME]) {
		const id =
			Math.random().toString(36).substring(2, 15) +
			Math.random().toString(36).substring(2, 15);
		const key =
			Math.random().toString(36).substring(2, 15) +
			Math.random().toString(36).substring(2, 15);
		setCookie(null, LOCAL_KEY_COOKIE_NAME, `${id}:${key}`, {
			maxAge: COOKIE_MAX_AGE,
			sameSite: 'strict',
			secure: true,
			path: '/',
		});
		return { id, key };
	}
	const [id, key] = cookies[LOCAL_KEY_COOKIE_NAME].split(':');
	return { id, key };
};

export const destroyLocalKey = () => {
	destroyCookie(null, LOCAL_KEY_COOKIE_NAME);
};
