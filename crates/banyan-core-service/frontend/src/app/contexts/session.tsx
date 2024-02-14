import { FC, ReactNode, createContext, useContext, useEffect, useState } from 'react';

import { LocalKey, UserData, getLocalKey, destroyLocalKey, getSessionKey, getUserData, getIsUserNew } from '@/app/utils/cookies';

export interface SessionState {
	localKey: LocalKey;
	userData: UserData | null;
	sessionKey: string | null;
	isUserNew: boolean;

	getLocalKey: () => LocalKey;
	destroyLocalKey: () => void;
	getUserData: () => UserData | null;
	getSessionKey: () => string | null
}

export const SessionContext = createContext<SessionState>({} as SessionState);

export const SessionProvider: FC<{ children: ReactNode }> = ({ children }) => {
	const [sessionState, setSessionState] = useState({
		localKey: getLocalKey(),
		userData: null,
		sessionKey: null,
		isUserNew: false,
		getLocalKey: getLocalKey,
		destroyLocalKey: destroyLocalKey,
		getSessionKey: getSessionKey,
		getUserData: getUserData
	} as SessionState);

	useEffect(() => {
		const userData = getUserData();
		const sessionKey = getSessionKey();
		const isUserNew = getIsUserNew();

		if (!userData || !sessionKey) {
			window.location.href = '/login';
			return;
		}

		setSessionState(prev => ({
			...prev,
			isUserNew,
			sessionKey,
			userData,
		}));
	}, []);

	return (
		<SessionContext.Provider value={sessionState}>
			{children}
		</SessionContext.Provider>
	);
};

export const useSession = () => useContext(SessionContext);