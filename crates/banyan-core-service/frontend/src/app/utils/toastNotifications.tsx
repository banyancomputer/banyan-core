import { ReactElement } from 'react';
import { toast } from 'react-toastify';

import { UploadFileProgress } from '@components/common/UploadFileProgress';

import { Close, Info, SuccessToastIcon } from '@static/images/common';
import { BrowserObject, Bucket } from '../types/bucket';

/** Defines toast notifications with message, toast type and theme. */
export class ToastNotifications {
    static notify(
        message: string,
        icon?: ReactElement | null,
        buttonMessage?: string,
        callback?: () => void,
    ) {
        toast.info(
            <div className="w-full flex items-center justify-between gap-2 text-xs">
                {message}
                {buttonMessage ?
                    <div
                        className="underline cursor-pointer"
                        onClick={() => {
                            callback && callback()
                            this.close();
                        }}
                    >
                        {buttonMessage}
                    </div>
                    :
                    null
                }
            </div>,
            {
                hideProgressBar: true,
                autoClose: 2000,
                icon: icon ? icon : <SuccessToastIcon />,
                closeButton: buttonMessage ? false : <Close />,
                bodyStyle: { gap: '8px', padding: '0', fontFamily: 'Inter', fontSize: '14px', fontWeight: 600 },
                style: { display: "flex", alignItems: 'center', padding: '12px 16px', borderRadius: '4px 4px 0 0', bottom: 0, margin: 0, width: '400px' },
            }
        );
    };

    static promise(pendingMessage: string, successMessage: string, icon: ReactElement, callback: any) {
        toast.promise(
            callback,
            {
                pending: {
                    render() { return pendingMessage; },
                },
                success: {
                    render() { return successMessage; },
                    icon: <span className="">{icon}</span>,
                },
            },
            {
                hideProgressBar: true,
                closeButton: () => <Close />,
                autoClose: false,
                bodyStyle: { gap: '8px', padding: '0', fontFamily: 'Inter', fontSize: '14px', fontWeight: 600 },
                style: { display: "flex", alignItems: 'center', padding: '12px 16px', borderRadius: '4px 4px 0 0', bottom: 0, margin: 0, width: '400px' },
            }
        );
    };

    static uploadProgress(bucket: Bucket, path: string[], folder?: BrowserObject) {
        toast.info(
            <UploadFileProgress
                bucket={bucket}
                path={path}
                folder={folder}
            />,
            {
                theme: "light",
                position: "bottom-right",
                icon: false,
                autoClose: false,
                closeButton: false,
                toastId: Date.now(),
                bodyStyle: { padding: '0', margin: '0', height: '100%', minHeight: 'unset !important', background: 'transparent' },
                style: { padding: '0', margin: '0', borderRadius: '4px', minHeight: 'unset !important', background: 'transparent' },
            });
    };

    static close() {
        toast.dismiss();
    };

    static error(message: string, buttonMessage?: string, callback?: () => void) {
        toast.error(
            <div className="w-full flex flex-col items-start justify-between gap-2 text-xs">
                {message}
            </div>,
            {
                icon: <Info />,
                closeButton: () => <Close />,
                hideProgressBar: true,
                autoClose: 2000,
                bodyStyle: { gap: '8px', padding: '0', fontFamily: 'Inter', fontSize: '14px', fontWeight: 600 },
                style: { display: "flex", alignItems: 'center', padding: '12px 16px', borderRadius: '4px 4px 0 0', bottom: 0, margin: 0, width: '400px' },
            }
        );
    }
};
