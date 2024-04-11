import React, { createContext, Dispatch, FC, ReactNode, SetStateAction, useContext, useState } from 'react';

export interface StateInterface {
	content: ReactNode | null;
	onBack: null | (() => void);
	mandatory: boolean;
	className?: string;
}

interface ContextState {
	modalState: StateInterface;
	setModalState: Dispatch<SetStateAction<StateInterface>>;
	openModal: (
		content: ReactNode,
		onBack?: null | (() => void),
		mandatory?: boolean,
		className?: string
	) => void;
	closeModal: () => void;
}

export const ModalContext = createContext<ContextState>({} as ContextState);

const initialState: StateInterface = {
	content: null,
	onBack: null,
	mandatory: false,
	className: '',
};

export const ModalProvider: FC<{ children: ReactNode }> = ({ children }) => {
	const [modalState, setModalState] = useState(initialState);

	const openModal = (
		content: ReactNode,
		onBack: null | (() => void) = null,
		mandatory: boolean = false,
		className?: string
	) => {
		setModalState({
			content,
			onBack,
			mandatory,
			className,
		});
	};

	const closeModal = () => {
		setModalState(initialState);
	};

	return (
		<ModalContext.Provider
			value={{ modalState, setModalState, openModal, closeModal }}
		>
			{children}
		</ModalContext.Provider>
	);
};

export const useModal = () => useContext(ModalContext);
