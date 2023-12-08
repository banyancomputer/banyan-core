import React, { useEffect, useRef } from 'react';
// @ts-ignore
import FilePreviewer from 'react-file-previewer';
import { useIntl } from 'react-intl';

import { Loader } from '../Loader';

import { SUPPORTED_EXTENSIONS, useFilePreview } from '@/app/contexts/filesPreview';

import { ArrowDown, ChevronUp } from '@static/images/common';
import { FileIcon } from '../FileIcon';

export const FilePreview = () => {
    const { bucket, file, files, path, openFile, closeFile } = useFilePreview();
    const { messages } = useIntl();
    const filePreviewRef = useRef<HTMLDivElement | null>(null);
    const fileExtension = [...file.name?.split('.')].pop();
    const isFileSupported = file.name ? SUPPORTED_EXTENSIONS.includes(fileExtension || '') : true;

    const close = (event: React.MouseEvent<HTMLDivElement | HTMLButtonElement>) => {
        if (!filePreviewRef.current!.contains(event.target as Node)) {
            closeFile();
        };
    };

    const openPrevious = () => {
        const selectedFileIndex = files.indexOf(file.name);
        if (!selectedFileIndex) return;
        openFile(bucket!, files[selectedFileIndex - 1], files, path);
    };

    const openNext = () => {
        const selectedFileIndex = files.indexOf(file.name);
        if (selectedFileIndex >= files.length - 1) return;
        openFile(bucket!, files[selectedFileIndex + 1], files, path);
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
        <>
            {(file.data || files.length || !isFileSupported || file.isLoading) &&
                <>
                    <button
                        onClick={close}
                        className="fixed left-12 top-10 flex items-center gap-3 z-40 text-white font-semibold"
                    >
                        <span className='rotate-90'>
                            <ArrowDown width="24px" height="24px" />
                        </span>
                        <FileIcon fileName={file.name} />
                        {`${file.name}`}
                    </button>
                    {files.indexOf(file.name) ?
                        <button
                            onClick={openPrevious}
                            className="fixed top-1/2 left-4 -translate-y-1/2 p-4 rounded-full bg-black text-white z-40 -rotate-90 transition-all hover:bg-gray-800"
                        >
                            <ChevronUp width="40px" height="40px" />
                        </button>
                        : null
                    }
                    {!(files.indexOf(file.name) === files.length - 1) ?
                        <button
                            onClick={openNext}
                            className="fixed top-1/2 right-4 -translate-y-1/2 p-4 rounded-full bg-black text-white z-40 rotate-90 transition-all hover:bg-gray-800"
                        >
                            <ChevronUp width="40px" height="40px" />
                        </button>
                        : null
                    }
                    <div
                        className={`fixed w-screen h-[105vh] flex ${isFileSupported ? 'items-start' : 'items-center'} justify-center py-16 pb-20 z-20 bg-slate-800 bg-opacity-80 backdrop-blur-sm overflow-scroll`}
                        onClick={close}
                    >
                        <div
                            className={`relative max-w-filePreview ${fileExtension === 'pdf' && 'w-filePreview'} ${!isFileSupported && 'pointer-events-none'} flex justify-center items-start `}
                            ref={filePreviewRef}
                        >
                            {file.isLoading ?
                                <Loader spinnerSize="50px" containerHeight="100vh" className="text-white" />
                                :
                                <>
                                    {
                                        isFileSupported && file.data ?
                                            <FilePreviewer
                                                hideControls
                                                file={{
                                                    url: file.data,
                                                    mimeType: `application/${[...file.name.split('.')].pop()}`,
                                                }}
                                            />
                                            :
                                            <div className="flex items-center text-white text-lg pointer-events-none">File is not supported for preview</div>
                                    }
                                </>
                            }
                        </div>
                    </div>
                </>
            }
        </>
    );
};
