import { FC, ReactNode, createContext, useContext, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { LocalKey, UserData, getLocalKey, destroyLocalKey, getSessionKey, getUserData } from '@/app/utils/cookies';

export interface SessionState {
	localKey: LocalKey;
	userData: UserData | null;
	sessionKey: string | null;

	getLocalKey: () => LocalKey;
	destroyLocalKey: () => void;
	getUserData: () => UserData | null;
	getSessionKey: () => string | null
}

export const SessionContext = createContext<SessionState>({} as SessionState);

export const SessionProvider: FC<{ children: ReactNode }> = ({ children }) => {
	const navigate = useNavigate();
	const [sessionState, setSessionState] = useState({
		localKey: getLocalKey(),
		userData: null,
		sessionKey: null,
		getLocalKey: getLocalKey,
		destroyLocalKey: destroyLocalKey,
		getSessionKey: getSessionKey,
		getUserData: getUserData
	} as SessionState);

	useEffect(() => {
		const userData = getUserData()
		const sessionKey = getSessionKey();

		if (!userData || !sessionKey) {
			window.location.href = '/login';
			return;
		}

		setSessionState({
			...sessionState,
			userData: getUserData(),
			sessionKey: getSessionKey(),
		})
	}, []);

	return (
		<SessionContext.Provider value={sessionState}>
			{children}
		</SessionContext.Provider>
	);
};

export const useSession = () => useContext(SessionContext);
