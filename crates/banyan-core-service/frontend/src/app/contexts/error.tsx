import { FC, ReactNode, createContext, useContext, useState } from 'react';

export class BannerError {
    constructor(
        public message: string = '',
        public action: { label: string, callback: () => void } | null = null,
        public canBeClosed: boolean = true,
    ) { };
};

interface ContextState {
    errors: BannerError[];
    setError: (error: BannerError) => void;
    closeError: (error: BannerError) => void;
};

export const ErrorContext = createContext<ContextState>({} as ContextState);

export const ErrorProvider: FC<{ children: ReactNode }> = ({ children }) => {
    const [errors, setErrors] = useState<BannerError[]>([]);

    const setError = (error: BannerError) => {
        setErrors(prev => [...prev, error]);
    };

    const closeError = (error: BannerError) => {
        setErrors(prev => prev.filter(existingError => existingError !== error));
    };

    return (
        <ErrorContext.Provider value={{ errors, setError, closeError }}>
            {children}
        </ErrorContext.Provider>
    );
};

export const useError = () => useContext(ErrorContext);