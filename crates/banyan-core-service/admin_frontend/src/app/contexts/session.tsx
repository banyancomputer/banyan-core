import {
	FC,
	ReactNode,
	createContext,
	useContext,
	useEffect,
	useState,
} from 'react';

import {
	LocalKey,
	UserData,
	getLocalKey,
	destroyLocalKey,
	getSessionKey,
} from '@/app/utils/cookies';
import { AdminClient } from '@/api/admin';

export interface SessionState {
	localKey: LocalKey;
	userData: UserData | null;
	sessionKey: string | null;

	getLocalKey: () => LocalKey;
	destroyLocalKey: () => void;
	getUser: () => UserData | null;
	getSessionKey: () => string | null;
}

export const SessionContext = createContext<SessionState>({} as SessionState);

export const SessionProvider: FC<{ children: ReactNode }> = ({ children }) => {
	const getUser = async () => {
		let userData = localStorage.getItem('userData');

		if (!userData) {
			const adminClient = new AdminClient();
			let userRes = await adminClient.getCurrentUser();
			userData = JSON.stringify(userRes);
			localStorage.setItem('userData', JSON.stringify(userData));
		}

		return JSON.parse(userData);

	}
	const [sessionState, setSessionState] = useState({
		localKey: getLocalKey(),
		userData: null,
		sessionKey: null,
		getLocalKey: getLocalKey,
		destroyLocalKey: destroyLocalKey,
		getSessionKey: getSessionKey,
	} as SessionState);

	useEffect(() => {
		(async () => {
			try {
				const userData = await getUser();
				const sessionKey = getSessionKey();

				if (!userData || !sessionKey) {
					window.location.href = '/login';
					return;
				}

				setSessionState({
					...sessionState,
					userData,
					sessionKey,
				});
			} catch (err) {
				console.error(err);
			}
		})();
	}, []);

	return (
		<SessionContext.Provider value={sessionState}>
			{children}
		</SessionContext.Provider>
	);
};

export const useSession = () => useContext(SessionContext);
