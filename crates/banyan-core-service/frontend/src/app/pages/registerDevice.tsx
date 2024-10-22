import { useParams } from 'react-router-dom';

import { SecondaryButton } from '@components/common/SecondaryButton';

import { publicPemWrap } from '@utils/index';
import { useAppDispatch, useAppSelector } from '@store/index';
import { approveDeviceApiKey } from '@store/tomb/actions';
import { unwrapResult } from '@reduxjs/toolkit';
import { ToastNotifications } from '../utils/toastNotifications';

// TODO: design must be handed down and implemented
const RegisterDevice = () => {
	const dispatch = useAppDispatch();
	const params = useParams();
	const spki = params.spki || '';
	const messages = useAppSelector(state => state.locales.messages.pages.registerDevice);

	// Perform all functions required to complete
	const completeRegistration = async () => {
		const pem = publicPemWrap(spki);
		try {
			unwrapResult(await dispatch(approveDeviceApiKey(pem)))
		} catch (error: any) {
			console.log(`error: ${error}`);
			ToastNotifications.error('failed to authorize new device!');
		};
	};

	return (
		<section className="py-9 px-4" id="buckets">
			<div className="mb-4 flex w-full justify-between items-center">
				<h2 className="text-xl font-semixld">
					Approve new device key?
				</h2>
				<div className="w-modal flex flex-col gap-8" >
					<div>
						<h4 className="text-m font-semibold">{`${messages.approveAccess}`}</h4>
						<p className="mt-2 text-text-600">
							{`${messages.wantToApproveAccess}?`}
						</p>
					</div>
					<div className="mt-3 flex items-center gap-3 text-xs" >
						<SecondaryButton
							className="w-1/2"
							action={() => window.location.href = '/'}
							text={`${messages.cancel}`}
						/>
						<button
							className="btn-secondary w-1/2 py-3 px-4"
						>
							{`${messages.cancel}`}
						</button>
						<button
							className="btn-primary w-1/2 py-3 px-4"
							onClick={completeRegistration}
						>{`${messages.approveAccess}`}</button>
					</div>
				</div>
			</div>
		</section>
	);
};

export default RegisterDevice;
