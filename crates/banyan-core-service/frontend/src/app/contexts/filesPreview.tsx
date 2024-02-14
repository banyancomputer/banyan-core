import React, { FC, ReactNode, createContext, useContext, useEffect, useState } from 'react';
import mime from 'mime';
import { useIntl } from 'react-intl';

import { useTomb } from './tomb';
import { BrowserObject, Bucket } from '@/app/types/bucket';
import { ToastNotifications } from '../utils/toastNotifications';
import { SUPPORTED_EXTENSIONS, fileTypes } from "@app/types/filesPreview";

interface OpenedFile {
    objectUrl: string;
    blob: File | null;
    fileType: string,
    isLoading: boolean;
    name: string;
    browserObject: BrowserObject | null;
};

const initialOpenedFileState: OpenedFile = {
    objectUrl: '',
    name: '',
    blob: null,
    fileType: '',
    isLoading: false,
    browserObject: null
};

class FilePreviewState {
    public file: OpenedFile = initialOpenedFileState;
    public files: BrowserObject[] = [];
    public bucket: Bucket | null = null;
    public path: string[] = [];
    public parrentFolder: BrowserObject | undefined = undefined;
    openFile: (bucket: Bucket, file: BrowserObject, files: BrowserObject[], path: string[], parrentFolder?: BrowserObject) => void = () => { };
    openNext: () => void = () => { };
    openPrevious: () => void = () => { };
    closeFile: () => void = () => { };
};

export const FilePreviewContext = createContext<FilePreviewState>({} as FilePreviewState);

export const FilePreviewProvider: FC<{ children: ReactNode }> = ({ children }) => {
    const [previewState, setPsreviewState] = useState<FilePreviewState>(new FilePreviewState());
    const { bucket, file, files, path, parrentFolder } = previewState;
    const { getFile } = useTomb();
    const { messages } = useIntl();

    const openFile = async (bucket: Bucket, file: BrowserObject, files: BrowserObject[], path: string[], parrentFolder?: BrowserObject) => {
        if (!file) return;

        setPsreviewState(prev => ({ ...prev, files, file: { ...initialOpenedFileState, name: file.name, browserObject: file }, bucket, path, parrentFolder }));

        const fileExtension = [...file.name.split('.')].pop() || '';
        const isFileSupported = SUPPORTED_EXTENSIONS.some((element, index) => {
            const result = element.includes(fileExtension);
            result && setPsreviewState(prev => ({ ...prev, file: { ...prev.file, browserObject: file, fileType: fileTypes[index] } }));
            return result;
        });

        if (!isFileSupported) {
            return;
        };

        try {
            setPsreviewState(prev => ({ ...prev, file: { ...prev.file, isLoading: true } }));
            const arrayBuffer = await getFile(bucket, path, file.name);
            const blob = new File([arrayBuffer], file.name, { type: mime.getType(fileExtension) || '' });
            const objectUrl = URL.createObjectURL(blob);
            setPsreviewState(prev => ({ ...prev, file: { ...prev.file, objectUrl, isLoading: false } }));
        } catch (error: any) {
            ToastNotifications.error('Failed to load file', `${messages.tryAgain}`, () => openFile(bucket, file, files, path));
            setPsreviewState(prev => ({ ...prev, file: initialOpenedFileState }));
        }
    };

    const openNext = () => {
        const selectedFileIndex = files.map(file => file.name).indexOf(file.name);
        openFile(bucket!, files[selectedFileIndex + 1], files, path, parrentFolder);
    };

    const openPrevious = () => {
        const selectedFileIndex = files.map(file => file.name).indexOf(file.name);
        openFile(bucket!, files[selectedFileIndex - 1], files, path, parrentFolder);
    };

    const closeFile = () => {
        setPsreviewState(new FilePreviewState());
    };

    useEffect(() => {
        const listener = (event: KeyboardEvent) => {
            const selectedFileIndex = files.map(file => file.name).indexOf(file.name);
            if (event.code === 'ArrowLeft' && selectedFileIndex) {
                openPrevious();
            } else if (event.code === 'ArrowRight' && (selectedFileIndex < files.length - 1)) {
                openNext();
            }
        };

        document.addEventListener('keydown', listener);

        return () => {
            document.removeEventListener('keydown', listener);
        }
    }, [previewState.file.name]);

    return (
        <FilePreviewContext.Provider value={{ ...previewState, openFile, openNext, openPrevious, closeFile }}>
            {children}
        </FilePreviewContext.Provider>
    );
};

export const useFilePreview = () => useContext(FilePreviewContext);
