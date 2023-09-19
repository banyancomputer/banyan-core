import React, { useRef } from 'react';
//@ts-ignore
import FilePreviewer from 'react-file-previewer';
import { FiArrowLeft } from 'react-icons/fi';
import { useIntl } from 'react-intl';

import { useFilePreview } from '@/contexts/filesPreview';

export const FilePreview = () => {
    const { file, closeFile } = useFilePreview();
    const { messages } = useIntl();
    const filePreviewRef = useRef<HTMLDivElement | null>(null);

    const close = (event: React.MouseEvent<HTMLDivElement | HTMLButtonElement>) => {
        if (!filePreviewRef.current!.contains(event.target as Node)) {
            closeFile();
        };
    };

    return (
        <>
            {file.data &&
                <div
                    className="absolute w-screen h-screen bg flex justify-center py-24 z-10 bg-slate-800 bg-opacity-80 backdrop-blur-sm overflow-scroll"
                    onClick={close}
                >
                    <button
                        onClick={close}
                        className="absolute left-12 top-20 flex items-center gap-3 text-white font-semibold"
                    >
                        <FiArrowLeft size="24px" />
                        {`${messages.backToFiles}`}
                    </button>
                    <div
                        className="relative max-w-filePreview w-full"
                        ref={filePreviewRef}
                    >
                        <FilePreviewer
                            hideControls
                            file={{
                                url: file.data,
                                mimeType: `application/${file.name.split('.')[1]}`,
                            }}
                        />
                    </div>
                </div>
            }
        </>
    )
}
