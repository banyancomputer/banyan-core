import React, { useRef } from 'react';
// @ts-ignore
import FilePreviewer from 'react-file-previewer';
import { useIntl } from 'react-intl';

import { Loader } from '../Loader';

import { SUPPORTED_EXTENSIONS, useFilePreview } from '@/app/contexts/filesPreview';

import { ArrowDown } from '@static/images/common';

export const FilePreview = () => {
    const { file, closeFile } = useFilePreview();
    const { messages } = useIntl();
    const filePreviewRef = useRef<HTMLDivElement | null>(null);
    const fileExtension = [...file.name.split('.')].pop();
    const isFileSupported = file.name ? SUPPORTED_EXTENSIONS.includes(fileExtension || '') : true;

    const close = (event: React.MouseEvent<HTMLDivElement | HTMLButtonElement>) => {
        if (!filePreviewRef.current!.contains(event.target as Node)) {
            closeFile();
        };
    };

    return (
        <>
            {(file.data || !isFileSupported || file.isLoading) &&
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
                    <div
                        className={`fixed w-screen h-screen flex ${isFileSupported ? 'items-start' : 'items-center'} justify-center py-10 z-20 bg-slate-800 bg-opacity-80 backdrop-blur-sm overflow-scroll`}
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
