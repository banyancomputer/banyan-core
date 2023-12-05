import React, { useEffect, useRef, useState } from 'react';
// @ts-ignore
import FilePreviewer from 'react-file-previewer';
import { useIntl } from 'react-intl';

import { Loader } from '../Loader';

import { SUPPORTED_EXTENSIONS, useFilePreview } from '@/app/contexts/filesPreview';

import { ArrowDown, ChevronUp } from '@static/images/common';

export const FilePreview = () => {
    const { bucket, file, files, path, openFile, closeFile } = useFilePreview();
    const { messages } = useIntl();
    const filePreviewRef = useRef<HTMLDivElement | null>(null);
    const fileExtension = [...file.name.split('.')].pop();
    const isFileSupported = file.name ? SUPPORTED_EXTENSIONS.includes(fileExtension || '') : true;
    const [selectedFileIndex, setSelectedFileIndex] = useState(0);

    const close = (event: React.MouseEvent<HTMLDivElement | HTMLButtonElement>) => {
        if (!filePreviewRef.current!.contains(event.target as Node)) {
            closeFile();
        };
    };

    const openPrevious = () => {
        openFile(bucket!, files[selectedFileIndex - 1], files, path);
    };

    const openNext = () => {
        openFile(bucket!, files[selectedFileIndex + 1], files, path);
    };

    useEffect(() => {
        const listener = (event: KeyboardEvent) => {
            if(file.isLoading) {
                document.removeEventListener('keydown', listener);
                return;
            }
            if (event.code === 'ArrowLeft' && selectedFileIndex) {
                document.removeEventListener('keydown', listener);
                openPrevious();
            } else if (event.code === 'ArrowRight' && !(selectedFileIndex === files.length - 1)) {
                document.removeEventListener('keydown', listener);
                openNext();
            }
        };

        document.addEventListener('keydown', listener);
    }, [selectedFileIndex, files.length, file.isLoading]);

    useEffect(() => {
        if (!file.name) return;

        setSelectedFileIndex(files.indexOf(file.name));
    }, [file.name, files]);

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
                        {`${messages.backToFiles}`}
                    </button>
                    {selectedFileIndex ?
                        <button
                            onClick={openPrevious}
                            className="fixed top-1/2 left-4 -translate-y-1/2 p-4 rounded-full bg-black text-white z-40 -rotate-90 transition-all hover:bg-gray-800"
                        >
                            <ChevronUp width="40px" height="40px" />
                        </button>
                        : null
                    }
                    {!(selectedFileIndex === files.length - 1) ?
                        <button
                            onClick={openNext}
                            className="fixed top-1/2 right-4 -translate-y-1/2 p-4 rounded-full bg-black text-white z-40 rotate-90 transition-all hover:bg-gray-800"
                        >
                            <ChevronUp width="40px" height="40px" />
                        </button>
                        : null
                    }
                    <div
                        className={`fixed w-screen h-[105vh] flex ${isFileSupported ? 'items-start' : 'items-center'} justify-center py-10 pb-20 z-20 bg-slate-800 bg-opacity-80 backdrop-blur-sm overflow-scroll`}
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
                                        isFileSupported ?
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
