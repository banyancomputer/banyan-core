import React, { FC, ReactNode, createContext, useContext, useEffect, useState } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';

import { useTomb } from './tomb';
import { BrowserObject, Bucket } from '@/app/types/bucket';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useModal } from './modals';
import { HardStorageLimit } from '../components/common/Modal/HardStorageLimit';
import { BannerError, setError } from '../store/errors/slice';
import { useAppDispatch, useAppSelector } from '../store';
import { SubscriptionPlanModal } from '../components/common/Modal/SubscriptionPlanModal';
import { FILE_SIZE_LIMIT } from '../utils/storage';

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
    const { hardStorageLimit, seePricingPage, softStorageLimit, contactSales } = useAppSelector(state => state.locales.messages.contexts.fileUpload);
    const { fileSizeExceeded } = useAppSelector(state => state.locales.messages.contexts.fileUpload);
    const dispatch = useAppDispatch();
    const [files, setFiles] = useState<UploadingFile[]>([]);
    const { openModal } = useModal();
    const [selectedBucket, setSelectedBucket] = useState<Bucket | null>(null);
    const [selectedPath, setSelectedPath] = useState<string[]>([]);
    const [selectedFolder, setSelectedFolder] = useState<BrowserObject | null>(null);
    const location = useLocation();
    const navigate = useNavigate();

    const uploadFiles = async (bucket: Bucket, path: string[], folder?: BrowserObject) => {
        if (files.some(file => file.file.size > FILE_SIZE_LIMIT)) {
            ToastNotifications.error(fileSizeExceeded);

            return;
        };

        ToastNotifications.uploadProgress();
        setSelectedBucket(bucket);
        setSelectedPath(path);
        setSelectedFolder(folder || null);

        if (!location.pathname.includes('drive')) {
            navigate(`/drive/${bucket.id}`);
        };

        for (const file of files) {
            try {
                if (file.file.size > storageUsage.softLimit - storageUsage.usage) {
                    setFiles(prev => prev.map(file => file.status === 'pending' ? { ...file, status: 'failed' } : file));
                    file.file.size > storageUsage.hardLimit - storageUsage.usage ?
                        dispatch(setError(new BannerError(hardStorageLimit, { callback: () => { window.location.href = 'mailto:tim@banyan.computer' }, label: contactSales })))
                        :
                        dispatch(setError(new BannerError(softStorageLimit, { callback: () => { openModal(<SubscriptionPlanModal />) }, label: seePricingPage })))
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
        if ((file.file.size > storageUsage.softLimit - storageUsage.usage) || (file.file.size > storageUsage.hardLimit - storageUsage.usage)) { return };

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

    useEffect(() => {

    }, [])

    return (
        <FilesUploadContext.Provider value={{ files, selectedBucket, selectedFolder, selectedPath, deleteFromUploadList, retryUpload, setFiles, uploadFiles }}>
            {children}
        </FilesUploadContext.Provider>
    );
};

export const useFilesUpload = () => useContext(FilesUploadContext);
