import { ToastContainer, ToastPosition } from 'react-toastify';

/** Custom component for notifications. */
export const Notifications: React.FC = () => {
	/** Describes notification position */
	const POSITION: ToastPosition = 'bottom-right';

	return (
		<ToastContainer
			position={POSITION}
			hideProgressBar
			limit={1}
			pauseOnFocusLoss
			pauseOnHover
		/>
	);
};
