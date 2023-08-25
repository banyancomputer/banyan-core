import React, { Dispatch, FC, ReactElement, ReactNode, SetStateAction, createContext, useContext, useState } from 'react';

export interface StateInterface {
    content: ReactNode | null;
}

interface ContextState {
    modalState: StateInterface;
    setModalState: Dispatch<SetStateAction<StateInterface>>;
    openModal: (content: ReactNode) => void;
    closeModal: () => void;
}

export const ModalContext = createContext<ContextState>({} as ContextState);

const initialState: StateInterface = {
    content: null,
};

export const ModalProvider: FC<{ children: ReactNode }> = ({ children }) => {
    const [modalState, setModalState] = useState(initialState);

    const openModal = (content: ReactNode) => {
        setModalState({
            content,
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
