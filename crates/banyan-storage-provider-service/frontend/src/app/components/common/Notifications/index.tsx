import { ToastContainer, ToastPosition } from 'react-toastify';

/** Custom component for notifications. */
export const Notifications: React.FC = () => {

    return <ToastContainer
        position="top-right"
        hideProgressBar
        limit={1}
        pauseOnFocusLoss
        pauseOnHover
    />;
};
