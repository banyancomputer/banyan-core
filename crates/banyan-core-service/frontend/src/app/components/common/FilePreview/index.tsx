import React from 'react';
// @ts-ignore
import FilePreviewer from 'react-file-previewer';

import { Loader } from '@components/common/Loader';
import { FileIcon } from '@components/common/FileIcon';

import { useFilePreview } from '@/app/contexts/filesPreview';

import { ArrowDown } from '@static/images/common';
import { PreviewArrow } from './Arrow';

export const FilePreview = () => {
    const { file, files, openNext, openPrevious, closeFile } = useFilePreview();

    const close = () => {
        closeFile();
    };

    const getPreviewTag = (data: string, type: string) => {
        switch (type) {
            case 'audio':
                return <audio
                    src={data}
                    controls
                    className="rounded-2xl"
                    onClick={event => event.stopPropagation()}
                />;
            case 'video':
                return <video
                    src={data}
                    controls
                    className="max-w-filePreview max-h-full object-contain rounded-2xl"
                    onClick={event => event.stopPropagation()}
                />;
            case 'image':
                return <img
                    src={data}
                    className="max-w-filePreview max-h-full object-contain rounded-2xl"
                    onClick={event => event.stopPropagation()}
                />;
            case 'document':
                return <div className='w-filePreview max-w-filePreview' onClick={event => event.stopPropagation()}>
                    <FilePreviewer
                        hideControls
                        file={{
                            url: file.data,
                            mimeType: `application/${[...file.name.split('.')].pop()}`,
                        }}
                    />
                </div>
            default:
                return <div className="flex items-center text-white text-lg pointer-events-none">File is not supported for preview</div>
        };
    };

    return (
        <>
            {file.name &&
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
                    <PreviewArrow
                        action={openPrevious}
                        isVisible={!!files.indexOf(file.name)}
                        className='left-4 -rotate-90'
                    />
                    <PreviewArrow
                        action={openNext}
                        isVisible={!(files.indexOf(file.name) === files.length - 1)}
                        className='right-4 rotate-90'
                    />
                    <div
                        className={`fixed w-screen h-[105vh] flex ${file.fileType === 'document' ? 'items-start' : 'items-center'} justify-center py-16 pb-20 z-20 bg-slate-800 bg-opacity-80 backdrop-blur-sm overflow-scroll`}
                        onClick={close}
                    >
                        {file.isLoading ?
                            <Loader spinnerSize="50px" containerHeight="80vh" className="text-white" />
                            :
                            <>
                                {
                                    file.data &&
                                    getPreviewTag(file.data, file.fileType)
                                }
                            </>
                        }
                    </div>
                </>
            }
        </>
    );
};
