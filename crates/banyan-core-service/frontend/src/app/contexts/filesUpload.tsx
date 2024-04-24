import React, { FC, ReactNode, createContext, useContext, useEffect, useState } from 'react';

import { useTomb } from './tomb';
import { BrowserObject, Bucket } from '@/app/types/bucket';
import { useModal } from './modals';
import { BannerError, setError } from '../store/errors/slice';
import { useAppDispatch, useAppSelector } from '../store';
import { SubscriptionPlanModal } from '../components/common/Modal/SubscriptionPlanModal';
import { FILE_SIZE_LIMIT } from '@app/utils/storage';
import { ToastNotifications } from '@app/utils/toastNotifications';

export interface UploadingFile { file: File; status: "pending" | "uploading" | "success" | "failed" };
interface FilesUploadState {
    files: UploadingFile[];
    deleteFromUploadList: (file: UploadingFile) => void;
    retryUpload: (file: UploadingFile, bucket: Bucket, path: string[], folder?: BrowserObject) => void;
    setFiles: React.Dispatch<React.SetStateAction<UploadingFile[]>>;
    uploadFiles: (files: FileList, bucket: Bucket, path: string[], folder?: BrowserObject) => void;
};

export const FilesUploadContext = createContext<FilesUploadState>({} as FilesUploadState);

export const FileUploadProvider: FC<{ children: ReactNode }> = ({ children }) => {
    const { storageUsage, storageLimits, uploadFile } = useTomb();
    const { hardStorageLimit, seePricingPage, softStorageLimit, contactSales } = useAppSelector(state => state.locales.messages.contexts.fileUpload);
    const { fileSizeExceeded } = useAppSelector(state => state.locales.messages.contexts.fileUpload);
    const dispatch = useAppDispatch();
    const [files, setFiles] = useState<UploadingFile[]>([]);
    const { openModal } = useModal();

    const uploadFiles = async (fileList: FileList, bucket: Bucket, path: string[], folder?: BrowserObject) => {
        const files: UploadingFile[] = Array.from(fileList).map(file => ({ file, status: 'pending' }));

        if (files.some(file => file.file.size > FILE_SIZE_LIMIT)) {
            ToastNotifications.error(fileSizeExceeded);

            return;
        };
        setFiles(files);

        ToastNotifications.uploadProgress(bucket, path, folder);

        for (const file of files) {
            try {
                if (file.file.size > storageLimits.softLimit - storageUsage.hotStorage) {
                    setFiles(prev => prev.map(file => file.status === 'pending' ? { ...file, status: 'failed' } : file));
                    file.file.size > storageLimits.hardLimit - storageUsage.hotStorage ?
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
                console.log('upload error', error);
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

    const retryUpload = async (file: UploadingFile, bucket: Bucket, path: string[], folder?: BrowserObject) => {
        if ((file.file.size > storageLimits.softLimit - storageUsage.hotStorage) || (file.file.size > storageLimits.hardLimit - storageUsage.hotStorage)) { return };

        try {
            const arrayBuffer = await file.file.arrayBuffer();
            file.status = 'uploading';
            setFiles(prev => [...prev]);
            await uploadFile(bucket!, path, file.file.name, arrayBuffer, folder!);
            file.status = 'success';
            setFiles(prev => [...prev]);
        } catch (error: any) {
            console.log('upload retry error', error);
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
        <FilesUploadContext.Provider value={{ files, deleteFromUploadList, retryUpload, setFiles, uploadFiles }}>
            {children}
        </FilesUploadContext.Provider>
    );
};

export const useFilesUpload = () => useContext(FilesUploadContext);
