import { ReactElement } from 'react';
import { IoMdClose } from 'react-icons/io';
import { toast } from 'react-toastify';

import { UploadFileProgress } from '@/app/components/common/UploadFileProgress';

/** Defines toast notifications with message, toast type and theme. */
export class ToastNotifications {
    static notify(
        message: string,
        icon?: ReactElement,
    ) {
        toast.info(
            message,
            {
                theme: 'light',
                hideProgressBar: true,
                autoClose: 2000,
                icon: icon ? <span className="bg-button-primary p-2 rounded-full text-navigation-text">{icon}</span> : null,
                bodyStyle: { gap: '17px', fontFamily: 'Inter', fontSize: '14px', fontWeight: 600, color: 'black' },
                style: { padding: '16px', borderRadius: '12px', borderColor: '#7D89B0', borderWidth: '2px', width: '400px' },
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
                    icon: <span className="bg-button-primary p-2 rounded-full text-navigation-text">{icon}</span>,
                },
            },
            {
                theme: 'light',
                hideProgressBar: true,
                autoClose: false,
                bodyStyle: { gap: '17px', fontFamily: 'Inter', fontSize: '14px', fontWeight: 600, color: 'black' },
                style: { padding: '16px', borderRadius: '12px', borderColor: 'var(--darken-border)', borderWidth: '2px', width: '400px' },
            }
        );
    };

    static uploadProgress() {
        toast.info(
            <UploadFileProgress />,
            {
                icon: false,
                autoClose: false,
                closeButton: false,
                bodyStyle: { padding: '0', margin: '0', height: '100%' },
                style: { padding: '0', borderRadius: '12px 12px 0 0' },
            });
    };

    static close() {
        toast.dismiss();
    };

    static error(message: string, buttonMessage: string, callback: () => void) {
        toast.error(
            <div className="w-full flex flex-col items-start justify-between gap-2 text-xs">
                {message}
                <button
                    className="text-text-600"
                    onClick={callback}
                >
                    {buttonMessage}
                </button>
            </div>,
            {
                icon: <span className="bg-gray-200 p-2 rounded-full">
                    <IoMdClose size="20px" fill="#4A5578" />
                </span>,
                hideProgressBar: true,
                autoClose: 2000,
                bodyStyle: { gap: '17px', fontFamily: 'Inter', padding: '0 6px', fontSize: '14px', fontWeight: 600, color: 'black' },
                style: { padding: '16px', borderRadius: '12px', borderColor: '#7D89B0', borderWidth: '2px', width: '400px' },
            }
        );
    }
};