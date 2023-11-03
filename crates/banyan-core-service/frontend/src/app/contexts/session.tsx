import { EscrowedKeyMaterial } from '@app/lib/crypto/types';
import { SessionData } from '@app/types';
import React, { Dispatch, FC, ReactElement, ReactNode, SetStateAction, createContext, useContext, useState } from 'react';


export const SessionContext = createContext<SessionData>({} as SessionData);

const initialState: SessionData = {
    accountId: '',
    email: '',
    escrowedKey: {} as EscrowedKeyMaterial,
    image: '',
    locale: '',
    name: '',
    verified_email: false
};

export const SessionProvider: FC<{ children: ReactNode }> = ({ children }) => {
    const [sessionState, setSessionState] = useState(initialState);


    return (
        <SessionContext.Provider value={sessionState}>
            {children}
        </SessionContext.Provider>
    );
};

export const useSession = () => useContext(SessionContext);
