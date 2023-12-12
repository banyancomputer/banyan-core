import React, { FC, ReactNode, createContext, useContext, useEffect, useState } from 'react';

import { useTomb } from './tomb';
import { Bucket } from '@/app/types/bucket';

interface FilePreviewState {
    file: {
        name: string;
        data: string;
        fileType: string,
        isLoading: boolean;
    };
    files: string[];
    bucket: Bucket | null;
    openFile: (bucket: Bucket, file: string, files: string[], path: string[]) => void;
    openNext: () => void;
    openPrevious: () => void;
    closeFile: () => void;
};

const initialState = {
    name: '',
    data: '',
    fileType: '',
    isLoading: false,
};

export const SUPPORTED_AUDIO_EXTENSIONS = ['mp3', 'ogg', 'wav'];
export const SUPPORTED_DOCUMENT_EXTENSIONS = ['pdf'];
export const SUPPORTED_IMAGE_EXTENSIONS = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'svg', 'webp'];
export const SUPPORTED_VIDEO_EXTENSIONS = ['mp4', 'webm', 'ogg'];
export const SUPPORTED_EXTENSIONS = [SUPPORTED_AUDIO_EXTENSIONS, SUPPORTED_DOCUMENT_EXTENSIONS, SUPPORTED_IMAGE_EXTENSIONS, SUPPORTED_VIDEO_EXTENSIONS];
const fileTypes = ['audio', 'document', 'image', 'video'];
export const FilePreviewContext = createContext<FilePreviewState>({} as FilePreviewState);

export const FilePreviewProvider: FC<{ children: ReactNode }> = ({ children }) => {
    const [file, setFile] = useState(initialState);
    const [files, setFiles] = useState<string[]>([]);
    const [bucket, setBucket] = useState<Bucket | null>(null);
    const [path, setPath] = useState<string[]>([]);
    const { getFile } = useTomb();

    const openFile = async (bucket: Bucket, file: string, files: string[], path: string[]) => {
        if (!file) return;

        setFiles(files);
        setBucket(bucket);
        setPath(path);

        const fileExtension = [...file.split('.')].pop() || '';
        const isFileSupported = SUPPORTED_EXTENSIONS.some((element, index) => {
            const result = element.includes(fileExtension);
            result && setFile(prev => ({ ...prev, fileType: fileTypes[index] }));
            return result;
        });

        if (!isFileSupported) {
            return;
        };

        try {
            setFile(prev => ({ data: '', name: file, fileType: prev.fileType, isLoading: true }));
            const arrayBuffer = await getFile(bucket, path, file);
            const blob = new Blob([arrayBuffer], { type: 'application/octet-stream' });
            const data = await URL.createObjectURL(blob);
            setFile(prev => ({ data, name: file, fileType: prev.fileType, isLoading: false }));
        } catch (error: any) {
            setFile(initialState);
        }
    };

    const openNext = () => {
        const selectedFileIndex = files.indexOf(file.name);
        if (selectedFileIndex >= files.length - 1) return;
        openFile(bucket!, files[selectedFileIndex + 1], files, path);
    };

    const openPrevious = () => {
        const selectedFileIndex = files.indexOf(file.name);
        if (!selectedFileIndex) return;
        openFile(bucket!, files[selectedFileIndex - 1], files, path);
    };

    const closeFile = () => {
        setFile(initialState);
        setFiles([]);
    };

    useEffect(() => {
        const listener = (event: KeyboardEvent) => {
            if (file.isLoading) {
                document.removeEventListener('keydown', listener);
                return;
            }
            if (event.code === 'ArrowLeft') {
                document.removeEventListener('keydown', listener);
                openPrevious();
            } else if (event.code === 'ArrowRight') {
                document.removeEventListener('keydown', listener);
                openNext();
            }
        };

        document.addEventListener('keydown', listener);

        return () => {
            document.removeEventListener('keydown', listener);
        }
    }, [files, file]);

    return (
        <FilePreviewContext.Provider value={{ file, files, bucket, openFile, closeFile, openNext, openPrevious }}>
            {children}
        </FilePreviewContext.Provider>
    );
};

export const useFilePreview = () => useContext(FilePreviewContext);
