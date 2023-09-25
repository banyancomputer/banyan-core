import { Bucket, BucketFile } from '@/lib/interfaces/bucket';
import React, { Dispatch, FC, ReactElement, ReactNode, SetStateAction, createContext, useContext, useState } from 'react';
import { useTomb } from './tomb';


interface FilePreviewState {
    file: {
        name: string;
        data: string;
    };
    openFile: (bucket: Bucket, file: string, path: string[]) => void;
    closeFile: () => void;
}
const initialState = {
    name: '',
    data: ''
};

export const FilePreviewContext = createContext<FilePreviewState>({} as FilePreviewState);

export const FilePreviewProvider: FC<{ children: ReactNode }> = ({ children }) => {
    const [file, setFile] = useState(initialState);
    const { getFile } = useTomb();

    const openFile = async (bucket: Bucket, file: string, path: string[]) => {
        try {

            const reader = new FileReader();
            const arrayBuffer = await getFile(bucket, path, file);
            const blob = new Blob([arrayBuffer], { type: 'application/octet-stream' });

            reader.readAsDataURL(blob);
            reader.onload = function (event) {
                const result = event.target?.result as string;
                setFile({
                    data: result || '',
                    name: file
                });
            };
            reader.readAsDataURL(blob);
        } catch (error: any) { }
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
