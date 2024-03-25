import { Dispatch, FC, ReactNode, SetStateAction, createContext, useContext, useState } from 'react';

export interface StateInterface {
	content: ReactNode | null;
	onBack: null | (() => void);
	mandatory: boolean;
	closeButton: boolean;
	className?: string
}

interface ContextState {
	modalState: StateInterface;
	setModalState: Dispatch<SetStateAction<StateInterface>>;
	openModal: (content: ReactNode, onBack?: null | (() => void), mandatory?: boolean, className?: string, closeButton?: boolean,) => void;
	closeModal: () => void;
};

export const ModalContext = createContext<ContextState>({} as ContextState);

const initialState: StateInterface = {
	content: null,
	onBack: null,
	mandatory: false,
	closeButton: true,
	className: ''
};

export const ModalProvider: FC<{ children: ReactNode }> = ({ children }) => {
	const [modalState, setModalState] = useState(initialState);

	const openModal = (content: ReactNode, onBack: null | (() => void) = null, mandatory: boolean = false, className?: string, closeButton: boolean = true) => {
		setModalState({
			content,
			onBack,
			mandatory,
			closeButton,
			className
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
