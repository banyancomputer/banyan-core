import React, { Dispatch, FC, ReactElement, ReactNode, SetStateAction, createContext, useContext, useState } from 'react';


interface FilePreviewState {
    file: {
        name: string;
        data: string;
    };
    openFile: (file: ArrayBuffer, name: string) => void;
    closeFile: () => void;
}
const initialState = {
    name: '',
    data: ''
};

export const FilePreviewContext = createContext<FilePreviewState>({} as FilePreviewState);

export const FilePreviewProvider: FC<{ children: ReactNode }> = ({ children }) => {
    const [file, setFile] = useState(initialState);

    const openFile = (arrayBuffer: ArrayBuffer, name: string) => {
        const reader = new FileReader();
        const blob = new Blob([arrayBuffer], { type: 'application/octet-stream' });

        reader.readAsDataURL(blob);
        reader.onload = function (event) {
            const result = event.target?.result as string;
            setFile({
                data: result || '',
                name
            });
        };
        reader.readAsDataURL(blob);
    };

    const closeFile = () => {
        setFile(initialState);
    };

    return (
        <FilePreviewContext.Provider value={{ file, openFile, closeFile }}>
            {children}
        </FilePreviewContext.Provider>
    );
};

export const useFilePreview = () => useContext(FilePreviewContext);
