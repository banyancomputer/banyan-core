import { FC, ReactNode, createContext, useContext, useEffect, useState } from 'react';
import Tracker from '@openreplay/tracker';

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

const tracker = new Tracker({
	projectKey: process.env.TRACKER_PROJECT_KEY || '',
	ingestPoint: process.env.TRACKET_INGEST_POINT || '',
});

export const SessionProvider: FC<{ children: ReactNode }> = ({ children }) => {
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
		const userData = getUserData();
		const sessionKey = getSessionKey();

		if (!userData || !sessionKey) {
			window.location.href = '/login';
			return;
		}

		setSessionState({
			...sessionState,
			userData,
			sessionKey,
		})
	}, []);

	useEffect(() => {
		tracker.start();
	}, []);

	useEffect(() => {
		if (!sessionState.userData?.user.id) return;

		tracker.setUserID(sessionState.userData?.user.id);
	}, [sessionState.userData?.user.id]);

	return (
		<SessionContext.Provider value={sessionState}>
			{children}
		</SessionContext.Provider>
	);
};

export const useSession = () => useContext(SessionContext);
