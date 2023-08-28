import { ReactElement } from 'react';
import { Id, toast } from 'react-toastify';

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
                icon: icon ? <span className="bg-gray-200 p-2 rounded-full">{icon}</span> : null,
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
                    render() { return pendingMessage },
                },
                success: {
                    render() { return successMessage },
                    icon: <span className="bg-gray-200 p-2 rounded-full">{icon}</span>,
                },
            },
            {
                theme: 'light',
                hideProgressBar: true,
                autoClose: false,
                bodyStyle: { gap: '17px', fontFamily: 'Inter', fontSize: '14px', fontWeight: 600, color: 'black' },
                style: { padding: '16px', borderRadius: '12px', borderColor: '#7D89B0', borderWidth: '2px', width: '400px' },
            }
        );
    }
};
