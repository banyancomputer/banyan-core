import { Bucket, BrowserObject } from '@/app/types/bucket';
import React, { FC, ReactNode, createContext, useContext, useState } from 'react';
import { useTomb } from './tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';

export interface UploadingFile { file: File, isUploaded: boolean };
interface FilesUploadState {
    files: UploadingFile[];
    setFiles: React.Dispatch<React.SetStateAction<UploadingFile[]>>;
    uploadFiles: (bucket: Bucket, path: string[], folder?: BrowserObject) => void;
};

export const FilesUploadContext = createContext<FilesUploadState>({} as FilesUploadState);

export const FileUploadProvider: FC<{ children: ReactNode }> = ({ children }) => {
    const { uploadFile } = useTomb();
    const [files, setFiles] = useState<UploadingFile[]>([]);

    const uploadFiles = async (bucket: Bucket, path: string[], folder?: BrowserObject) => {
        let filesCopy = [...files];
        for (let file of files) {
            const arrayBuffer = await file.file.arrayBuffer();
            await uploadFile(bucket, path, file.file.name, arrayBuffer, folder);
            filesCopy = filesCopy.map(item => item.file.name === file.file.name ? ({ ...item, isUploaded: true }) : item);
            await setFiles(filesCopy);
        };
        ToastNotifications.close();
        setFiles([]);
    };

    return (
        <FilesUploadContext.Provider value={{ files, setFiles, uploadFiles }}>
            {children}
        </FilesUploadContext.Provider>
    );
};

export const useFilesUpload = () => useContext(FilesUploadContext);
