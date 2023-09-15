import { CreateSecretKeyModal } from '@/components/common/Modal/CreateSecretKeyModal';
import { EnterSecretKeyModal } from '@/components/common/Modal/EnterSecretKeyModal';
import React, { Dispatch, FC, ReactElement, ReactNode, SetStateAction, createContext, useContext, useState } from 'react';

export interface StateInterface {
    content: ReactNode | null;
    onBack: null | (() => void);
    mandatory: boolean
}

interface ContextState {
    modalState: StateInterface;
    setModalState: Dispatch<SetStateAction<StateInterface>>;
    openModal: (content: ReactNode, onBack?: null | (() => void)) => void;
    openEscrowModal: (escrowed: boolean) => void;
    closeModal: () => void;
}

export const ModalContext = createContext<ContextState>({} as ContextState);

const initialState: StateInterface = {
    content: null,
    onBack: null,
    mandatory: false
};

export const ModalProvider: FC<{ children: ReactNode }> = ({ children }) => {
    const [modalState, setModalState] = useState(initialState);

    const openModal = (content: ReactNode, onBack: null | (() => void) = null, mandatory: boolean = false) => {
        setModalState({
            content,
            onBack,
            mandatory
        });
    };

    const openEscrowModal = (escrowed: boolean) => {
        setModalState({
            content: escrowed ? <EnterSecretKeyModal /> : <CreateSecretKeyModal />,
            onBack: null,
            mandatory: true
        })
    };

    const closeModal = () => {
        setModalState(initialState);
    };

    return (
        <ModalContext.Provider value={{ modalState, setModalState, openModal, openEscrowModal, closeModal }}>
            {children}
        </ModalContext.Provider>
    );
};

export const useModal = () => useContext(ModalContext);
