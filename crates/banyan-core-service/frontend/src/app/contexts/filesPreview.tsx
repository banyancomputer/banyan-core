import React, { FC, ReactNode, createContext, useContext, useState } from 'react';

import { useTomb } from './tomb';
import { Bucket } from '@/app/types/bucket';

interface FilePreviewState {
    file: {
        name: string;
        data: string;
        isLoading: boolean;
    };
    files: string[];
    path: string[];
    bucket: Bucket | null;
    openFile: (bucket: Bucket, file: string, files: string[], path: string[]) => void;
    closeFile: () => void;
};

const initialState = {
    name: '',
    data: '',
    isLoading: false,
};

export const SUPPORTED_EXTENSIONS = ['pdf', 'gif', 'jpg', 'jpeg', 'png'];
export const FilePreviewContext = createContext<FilePreviewState>({} as FilePreviewState);

export const FilePreviewProvider: FC<{ children: ReactNode }> = ({ children }) => {
    const [file, setFile] = useState(initialState);
    const [files, setFiles] = useState<string[]>([]);
    const [bucket, setBucket] = useState<Bucket | null>(null)
    const [path, setPath] = useState<string[]>([])
    const { getFile } = useTomb();

    const openFile = async (bucket: Bucket, file: string, files: string[], path: string[]) => {
        setFiles(files);
        setBucket(bucket);
        setPath(path);
        if (!file) return;

        const isFileSupported = SUPPORTED_EXTENSIONS.includes([...file.split('.')].pop() || '');
        try {
            setFile({
                data: '',
                name: file,
                isLoading: false,
            });

            if (!isFileSupported) { return; }
            setFile(prev => ({ ...prev, isLoading: true }));

            const reader = new FileReader();
            const arrayBuffer = await getFile(bucket, path, file);
            const blob = new Blob([arrayBuffer], { type: 'application/octet-stream' });

            reader.readAsDataURL(blob);
            reader.onload = function (event) {
                const result = event.target?.result as string;
                setFile({
                    data: result || '',
                    name: file,
                    isLoading: false,
                });
            };
            reader.readAsDataURL(blob);
        } catch (error: any) {
            setFile(initialState);
        }
    };

    const closeFile = () => {
        setFile(initialState);
        setFiles([]);
    };

    return (
        <FilePreviewContext.Provider value={{ file, files, bucket, path, openFile, closeFile }}>
            {children}
        </FilePreviewContext.Provider>
    );
};

export const useFilePreview = () => useContext(FilePreviewContext);
