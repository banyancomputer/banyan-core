import { ToastContainer, ToastPosition } from 'react-toastify';

/** Custom component for notifications. */
export const Notifications: React.FC = () => {
    /** Describes notification position */
    const POSITION: ToastPosition = 'bottom-center';

    return <ToastContainer
        position={POSITION}
        hideProgressBar
        theme="dark"
        limit={1}
        pauseOnFocusLoss
        pauseOnHover
    />;
};
