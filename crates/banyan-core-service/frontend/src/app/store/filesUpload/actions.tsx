import { createAsyncThunk, unwrapResult } from "@reduxjs/toolkit";
import { RootState } from "@store/index";
import { BrowserObject, Bucket } from "@/app/types/bucket";
import { UploadingFile, setFiles, updateFileStatus } from "./slice";
import { FILE_SIZE_LIMIT } from "@/app/utils/storage";
import { ToastNotifications } from "@/app/utils/toastNotifications";
import { BannerError, setError } from "../errors/slice";
import { openModal } from "../modals/slice";
import { SubscriptionPlanModal } from '@components/common/Modal/SubscriptionPlanModal';
import { updateStorageUsageState, uploadFile } from "../tomb/actions";

export const uploadFiles = createAsyncThunk(
    'uploadFiles',
    async ({ fileList, bucket, path, folderLocation, folder }: { fileList: FileList, bucket: Bucket, path: string[], folderLocation: string[], folder?: BrowserObject }, { dispatch, getState }) => {
        const { tomb: { storageLimits, storageUsage }, locales } = getState() as RootState;
        const { contactSales, fileSizeExceeded, hardStorageLimit, softStorageLimit, seePricingPage } = locales.messages.contexts.fileUpload;

        const files: UploadingFile[] = Array.from(fileList).map(file => ({ file, status: 'pending' }));

        if (files.some(file => file.file.size > FILE_SIZE_LIMIT)) {
            ToastNotifications.error(fileSizeExceeded);

            return;
        };

        dispatch(setFiles(files));
        ToastNotifications.uploadProgress(bucket, path, folder);

        for (const file of files) {
            try {
                if (file.file.size > storageLimits.softLimit - storageUsage.hotStorage) {
                    file.status = 'failed';
                    dispatch(updateFileStatus({ file, status: 'failed' }));
                    file.file.size > storageLimits.hardLimit - storageUsage.hotStorage ?
                        dispatch(setError(new BannerError(hardStorageLimit, { callback: () => { window.location.href = 'mailto:tim@banyan.computer' }, label: contactSales })))
                        :
                        dispatch(setError(new BannerError(softStorageLimit, {
                            callback: () => {
                                dispatch(openModal({ content: <SubscriptionPlanModal /> }))
                            }, label: seePricingPage
                        })));
                };

                dispatch(updateFileStatus({ file, status: 'uploading' }));
                const arrayBuffer = await file.file.arrayBuffer();
                unwrapResult(await dispatch(uploadFile({ bucket: { ...bucket, mount: bucket.mount }, uploadPath: path, name: file.file.name, file: arrayBuffer, folder, folderLocation })));
                await dispatch(updateStorageUsageState());
                dispatch(updateFileStatus({ file, status: 'success' }));
            } catch (error: any) {
                console.log('upload error', error);
                dispatch(updateFileStatus({ file, status: 'failed' }));
                continue;
            }
        }
    }
);

export const retryUpload = createAsyncThunk(
    'retryUpload',
    async ({ file, bucket, path, folderLocation, folder }: { file: UploadingFile, bucket: Bucket, path: string[], folderLocation: string[], folder?: BrowserObject }, { dispatch, getState }) => {
        const { tomb: { storageLimits, storageUsage } } = getState() as RootState;
        if ((file.file.size > storageLimits.softLimit - storageUsage.hotStorage) || (file.file.size > storageLimits.hardLimit - storageUsage.hotStorage)) { return };

        try {
            const arrayBuffer = await file.file.arrayBuffer();
            dispatch(updateFileStatus({ file, status: 'uploading' }));
            unwrapResult(await dispatch(uploadFile({ bucket: { ...bucket, mount: bucket.mount }, uploadPath: path, name: file.file.name, file: arrayBuffer, folder, folderLocation })));
            await dispatch(updateStorageUsageState());
            dispatch(updateFileStatus({ file, status: 'success' }));
        } catch (error: any) {
            dispatch(updateFileStatus({ file, status: 'failed' }));
        }
    }
);
