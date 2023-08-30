import React, { Dispatch, FC, ReactElement, ReactNode, SetStateAction, createContext, useContext, useState } from 'react';

export interface StateInterface {
    content: ReactNode | null;
    onBack: null | (() => void);
}

interface ContextState {
    modalState: StateInterface;
    setModalState: Dispatch<SetStateAction<StateInterface>>;
    openModal: (content: ReactNode, onBack?: null | (() => void)) => void;
    closeModal: () => void;
}

export const ModalContext = createContext<ContextState>({} as ContextState);

const initialState: StateInterface = {
    content: null,
    onBack: null
};

export const ModalProvider: FC<{ children: ReactNode }> = ({ children }) => {
    const [modalState, setModalState] = useState(initialState);

    const openModal = (content: ReactNode, onBack: null | (() => void) = null) => {
        setModalState({
            content,
            onBack
        });
    };

    const closeModal = () => {
        setModalState(initialState);
    };

    return (
        <ModalContext.Provider value={{ modalState, setModalState, openModal, closeModal }}>
            {children}
        </ModalContext.Provider>
    );
};

export const useModal = () => useContext(ModalContext);
