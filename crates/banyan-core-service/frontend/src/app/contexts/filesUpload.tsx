import React, { FC, ReactNode, createContext, useContext, useState } from 'react';

import { useTomb } from './tomb';
import { BrowserObject, Bucket } from '@/app/types/bucket';
import { ToastNotifications } from '@/app/utils/toastNotifications';

export interface UploadingFile { file: File; status: "pending" | "uploading" | "success" | "failed" };
interface FilesUploadState {
    files: UploadingFile[];
    selectedBucket: Bucket | null;
    selectedPath: string[];
    selectedFolder: BrowserObject | null;
    deleteFromUploadList: (file: UploadingFile) => void;
    retryUpload: (file: UploadingFile) => void;
    setFiles: React.Dispatch<React.SetStateAction<UploadingFile[]>>;
    uploadFiles: (bucket: Bucket, path: string[], folder?: BrowserObject) => void;
};

export const FilesUploadContext = createContext<FilesUploadState>({} as FilesUploadState);

export const FileUploadProvider: FC<{ children: ReactNode }> = ({ children }) => {
    const { uploadFile } = useTomb();
    const [files, setFiles] = useState<UploadingFile[]>([]);
    const [selectedBucket, setSelectedBucket] = useState<Bucket | null>(null);
    const [selectedPath, setSelectedPath] = useState<string[]>([]);
    const [selectedFolder, setSelectedFolder] = useState<BrowserObject | null>(null);

    const uploadFiles = async (bucket: Bucket, path: string[], folder?: BrowserObject) => {
        setSelectedBucket(bucket);
        setSelectedPath(path);
        setSelectedFolder(folder || null);

        for (const file of files) {
            try {
                const arrayBuffer = await file.file.arrayBuffer();
                file.status = 'uploading';
                setFiles(prev => [...prev]);
                await uploadFile(bucket, path, file.file.name, arrayBuffer, folder);
                file.status = 'failed';
                setFiles(prev => [...prev]);
            } catch (error: any) {
                file.status = 'failed';
                setFiles(prev => [...prev]);
                continue;
            }
        };
        if (files.every(file => file.status === 'success')) {
            ToastNotifications.close();
            setFiles([]);
        }
    };

    const retryUpload = async (file: UploadingFile) => {
        try {
            const arrayBuffer = await file.file.arrayBuffer();
            file.status = 'uploading';
            setFiles(prev => [...prev]);
            await uploadFile(selectedBucket!, selectedPath, file.file.name, arrayBuffer, selectedFolder!);
            file.status = 'success';
            setFiles(prev => [...prev]);
        } catch (error: any) {
            file.status = 'failed';
            setFiles(prev => [...prev]);
        }
    };

    const deleteFromUploadList = (file: UploadingFile) => {
        setFiles(prev => prev.filter(uploadingFile => uploadingFile !== file));
    };

    return (
        <FilesUploadContext.Provider value={{ files, selectedBucket, selectedFolder, selectedPath, deleteFromUploadList, retryUpload, setFiles, uploadFiles }}>
            {children}
        </FilesUploadContext.Provider>
    );
};

export const useFilesUpload = () => useContext(FilesUploadContext);
