import { ReactElement } from 'react';
import { toast } from 'react-toastify';

/** Defines toast notifications with message, toast type and theme. */
export class ToastNotifications {
    /** Notifies user.
    * As default type uses error type, and default theme is colored. */
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
};
