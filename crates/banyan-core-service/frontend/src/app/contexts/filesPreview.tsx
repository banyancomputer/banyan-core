import React, { FC, ReactNode, createContext, useContext, useEffect, useState } from 'react';
import mime from 'mime';

import { useTomb } from './tomb';
import { BrowserObject, Bucket } from '@/app/types/bucket';
import { ToastNotifications } from '../utils/toastNotifications';
import { useIntl } from 'react-intl';

interface FileState {
    name: string;
    data: string;
    blob: File | null;
    fileType: string,
    isLoading: boolean;
    rawData: BrowserObject | null;
};

interface FilePreviewState {
    file: FileState;
    files: BrowserObject[];
    bucket: Bucket | null;
    path: string[];
    parrentFolder: BrowserObject | undefined;
    openFile: (bucket: Bucket, file: BrowserObject, files: BrowserObject[], path: string[], parrentFolder?: BrowserObject) => void;
    openNext: () => void;
    openPrevious: () => void;
    closeFile: () => void;
};

const initialState: FileState = {
    name: '',
    data: '',
    blob: null,
    fileType: '',
    isLoading: false,
    rawData: null
};

export const SUPPORTED_AUDIO_EXTENSIONS = ['mp3', 'ogg', 'wav'];
export const SUPPORTED_DOCUMENT_EXTENSIONS = ['pdf'];
export const SUPPORTED_IMAGE_EXTENSIONS = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'svg', 'webp'];
export const SUPPORTED_SPREADSHEET_EXTENSIONS = ['csv'];
export const SUPPORTED_VIDEO_EXTENSIONS = ['mp4', 'webm', 'ogg'];
export const SUPPORTED_EXTENSIONS = [SUPPORTED_AUDIO_EXTENSIONS, SUPPORTED_DOCUMENT_EXTENSIONS, SUPPORTED_IMAGE_EXTENSIONS, SUPPORTED_SPREADSHEET_EXTENSIONS, SUPPORTED_VIDEO_EXTENSIONS];
const fileTypes = ['audio', 'document', 'image', 'spreadsheet', 'video'];

export const FilePreviewContext = createContext<FilePreviewState>({} as FilePreviewState);

export const FilePreviewProvider: FC<{ children: ReactNode }> = ({ children }) => {
    const [file, setFile] = useState(initialState);
    const [files, setFiles] = useState<BrowserObject[]>([]);
    const [bucket, setBucket] = useState<Bucket | null>(null);
    const [parrentFolder, setParrentFolder] = useState<BrowserObject | undefined>(undefined);
    const [path, setPath] = useState<string[]>([]);
    const { getFile } = useTomb();
    const { messages } = useIntl();

    const openFile = async (bucket: Bucket, file: BrowserObject, files: BrowserObject[], path: string[], parrentFolder?: BrowserObject) => {
        if (!file) return;

        setFile({ ...initialState, name: file.name, rawData: file });
        setFiles(files);
        setBucket(bucket);
        setPath(path);
        setParrentFolder(undefined);

        const fileExtension = [...file.name.split('.')].pop() || '';
        const isFileSupported = SUPPORTED_EXTENSIONS.some((element, index) => {
            const result = element.includes(fileExtension);
            result && setFile(prev => ({ ...prev, fileType: fileTypes[index] }));
            return result;
        });

        if (!isFileSupported) {
            return;
        };

        try {
            setFile(prev => ({ ...prev, isLoading: true }));
            const arrayBuffer = await getFile(bucket, path, file.name);
            const blob = new File([arrayBuffer], file.name, { type: mime.getType(fileExtension) || '' });
            const data = URL.createObjectURL(blob);
            setFile(prev => ({ data, name: file.name, rawData: file, blob, fileType: prev.fileType, isLoading: false }));
        } catch (error: any) {
            ToastNotifications.error('Failed to load file', `${messages.tryAgain}`, () => openFile(bucket, file, files, path));
            setFile(initialState);
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
        setFile(initialState);
        setFiles([]);
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
    }, [file.name]);

    return (
        <FilePreviewContext.Provider value={{ file, files, path, bucket, parrentFolder, openFile, closeFile, openNext, openPrevious }}>
            {children}
        </FilePreviewContext.Provider>
    );
};

export const useFilePreview = () => useContext(FilePreviewContext);
