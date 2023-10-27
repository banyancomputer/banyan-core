import { NotificationBolt } from '@static/images';
import { toast } from 'react-toastify';


/** Defines toast notifications with message, toast type and theme. */
export class ToastNotifications {
    static notify(
        message: string,
    ) {
        toast.info(
            message,
            {
                theme: 'light',
                hideProgressBar: true,
                autoClose: 2000,
                closeButton: false,
                icon: <NotificationBolt />,
                bodyStyle: { gap: '17px', fontFamily: 'Inter', fontSize: '14px', padding: 0, fontWeight: 600, color: 'black' },
                style: { padding: '16px', borderRadius: '12px', borderWidth: '2px', width: '400px'},
            }
        );
    };

    static close() {
        toast.dismiss();
    };
};
