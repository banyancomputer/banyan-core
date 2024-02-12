import React, { FC, ReactNode, createContext, useContext, useState } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';

import { useTomb } from './tomb';
import { BrowserObject, Bucket } from '@/app/types/bucket';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useModal } from './modals';
import { HardStorageLimit } from '../components/common/Modal/HardStorageLimit';

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
    const { storageUsage, uploadFile } = useTomb();
    const [files, setFiles] = useState<UploadingFile[]>([]);
    const { openModal } = useModal();
    const [selectedBucket, setSelectedBucket] = useState<Bucket | null>(null);
    const [selectedPath, setSelectedPath] = useState<string[]>([]);
    const [selectedFolder, setSelectedFolder] = useState<BrowserObject | null>(null);
    const location = useLocation();
    const navigate = useNavigate();

    const uploadFiles = async (bucket: Bucket, path: string[], folder?: BrowserObject) => {
        setSelectedBucket(bucket);
        setSelectedPath(path);
        setSelectedFolder(folder || null);

        if (!location.pathname.includes('drive')) {
            navigate(`/drive/${bucket.id}`);
        };

        for (const file of files) {
            try {
                if (file.file.size > storageUsage.hardLimit - storageUsage.usage) {
                    setFiles(prev => prev.map(file => file.status === 'pending' ? { ...file, status: 'failed' } : file));
                    openModal(<HardStorageLimit />, null, false, 'p-0', false);
                    return;
                };
                const arrayBuffer = await file.file.arrayBuffer();
                file.status = 'uploading';
                setFiles(prev => [...prev]);
                await uploadFile(bucket, path, file.file.name, arrayBuffer, folder);
                file.status = 'success';
                setFiles(prev => [...prev]);
            } catch (error: any) {
                file.status = 'failed';
                setFiles(prev => [...prev]);
                continue;
            }
        };
        if (files.every(file => file.status === 'success')) {
            setTimeout(() => {
                ToastNotifications.close();
                setFiles([]);
            }, 3000);
        };
    };

    const retryUpload = async (file: UploadingFile) => {
        if (file.file.size > storageUsage.hardLimit - storageUsage.usage) { return };
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
        };
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
