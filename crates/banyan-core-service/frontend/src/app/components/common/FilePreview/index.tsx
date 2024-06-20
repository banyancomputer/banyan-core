import { useEffect } from 'react';
import { unwrapResult } from '@reduxjs/toolkit';

import { Loader } from '@components/common/Loader';
import { SpreadsheetViewer } from '@components/common/FilePreview/SpreadsheetViewer';
import { FilePreviewActions } from '@components/common/FilePreview/Actions';
import { PreviewArrow } from '@components/common/FilePreview/Arrow';
import { ShareFileModal } from '@components/common/Modal/ShareFileModal';

import { openModal } from '@store/modals/slice';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useAppDispatch, useAppSelector } from '@store/index';

import { Close, Done, DownloadAlternative, Upload } from '@static/images/common';
import { getFile, shareFile } from '@store/tomb/actions';
import { closeFile, openFile } from '@store/filePreview/slice';
import { loadFilePreview } from '@store/filePreview/actions';

export const FilePreview = () => {
    const dispatch = useAppDispatch();
    const messages = useAppSelector(state => state.locales.messages.coponents.common.filePreview);
    const { file, files, bucket, path, parrentFolder } = useAppSelector(state => state.filePreview);

    const close = () => {
        dispatch(closeFile());
    };

    const downloadFile = async () => {
        try {
            await ToastNotifications.promise(`${messages.downloading}...`, `${messages.fileWasDownloaded}`, <Done width="20px" height="20px" />,
                (async () => {
                    const link = document.createElement('a');
                    const arrayBuffer = unwrapResult(await dispatch(getFile({ bucket: bucket!, path, name: file.name })));
                    const blob = new Blob([arrayBuffer]);
                    const objectURL = URL.createObjectURL(blob);
                    link.href = objectURL;
                    link.download = file.name;
                    document.body.appendChild(link);
                    link.click();
                })()
            );
        } catch (error: any) {
            ToastNotifications.error('Failed to download file', `${messages.tryAgain}`, downloadFile);
        }
    };

    const share = async () => {
        try {
            const payload = unwrapResult(await dispatch(shareFile({ bucket: bucket!, path: [...path, file.name] })));
            const link = `${window.location.origin}/api/v1/share?payload=${payload}`;
            dispatch(openModal({ content: <ShareFileModal link={link} /> }));
        } catch (error: any) {
            ToastNotifications.error('Error while sharing file', `${messages.tryAgain}`, share);
        }
    };

    const openNext = () => {
        const selectedFileIndex = files.map(file => file.name).indexOf(file.name);
        dispatch(openFile({ bucket: bucket!, file: files[selectedFileIndex + 1], files, path, parrentFolder }));
    };

    const openPrevious = () => {
        const selectedFileIndex = files.map(file => file.name).indexOf(file.name);
        dispatch(openFile({ bucket: bucket!, file: files[selectedFileIndex - 1], files, path, parrentFolder }));
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
            case 'spreadsheet':
                return <SpreadsheetViewer data={file.blob!} />;
            case 'document':
                return <div className="w-filePreview max-w-filePreview h-full" onClick={event => event.stopPropagation()}>
                    <object
                        data={`${data}#toolbar=0`}
                        type={file.mimeType}
                        className="w-full h-full rounded-xl"
                    />
                </div>;
            default:
                return <div className="flex items-center text-white text-lg pointer-events-none">File is not supported for preview</div>;
        };
    };


    useEffect(() => {
        if (!file.name || !file.fileType) return;

        dispatch(loadFilePreview());
    }, [file.name, file.fileType]);

    useEffect(() => {
        if (!file.name) return;

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
        <>
            {file.name &&
                <>
                    <div className="flex justify-between top-0 right-0 w-full  px-10 py-5">
                        <button
                            onClick={close}
                            className="flex items-center gap-3 z-40 text-white font-semibold"
                        >
                            <Close width="24px" height="24px" />
                            {`${file.name}`}
                        </button>
                        <div className="flex items-center gap-4">
                            <div
                                className="text-white z-40 cursor-pointer"
                                onClick={downloadFile}
                            >
                                <DownloadAlternative width="24px" height="24px" />
                            </div>
                            <FilePreviewActions
                                bucket={bucket!}
                                file={file.browserObject!}
                                parrentFolder={parrentFolder!}
                                path={path}
                            />
                            <div
                                className="flex items-center gap-2 px-2 py-1 h-10 bg-bucket-actionsBackground text-bucket-actionsText z-40 rounded cursor-pointer"
                                onClick={share}
                            >
                                <Upload width="20px" height="20px" />
                                {`${messages.shareFile}`}
                            </div>
                        </div>
                    </div>
                    <div
                        className={`fixed w-screen h-[105vh] flex ${file.fileType === 'document' || file.fileType === 'spreadsheet' ? 'items-start' : 'items-center'} justify-center py-16 pb-20 z-20 bg-[#0d0d0dcc] overflow-scroll`}
                        onClick={close}
                    >
                        <PreviewArrow
                            action={openPrevious}
                            isVisible={!!files.map(file => file.name).indexOf(file.name)}
                            className="rotate-90 left-20"
                        />
                        <PreviewArrow
                            action={openNext}
                            isVisible={!(files.map(file => file.name).indexOf(file.name) === files.length - 1)}
                            className="-rotate-90 right-20"
                        />
                        {file.isLoading ?
                            <Loader spinnerSize="50px" containerHeight="80vh" className="text-white" />
                            :
                            <>
                                {getPreviewTag(file.objectUrl, file.fileType)}
                            </>
                        }
                    </div>
                </>
            }
        </>
    );
};
