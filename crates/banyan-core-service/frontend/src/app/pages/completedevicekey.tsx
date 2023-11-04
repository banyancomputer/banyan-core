import React, { useEffect, useState } from 'react';
import { useIntl } from 'react-intl';
import { useParams } from 'react-router-dom';

import { useTomb } from '@/app/contexts/tomb';
import { useModal } from '@/app/contexts/modals';
import { b64UrlDecode } from '@/app/utils/b64';
import { hexFingerprintApiKeySpki, prettyFingerprintApiKeySpki, publicPemWrap } from '@/app/utils';
import { ClientApi } from '@/app/lib/api/auth';
import { DeviceApiKey } from '@/app/types';
import { useSession } from '@app/contexts/session';

const DeviceKeyApproval = () => {
	const api = new ClientApi();
	const { openModal } = useModal();
	const { messages } = useIntl();
	const { completeDeviceKeyRegistration } = useTomb();
	const searchParams = useParams();
	const urlSpki = searchParams.spki;
	const spki = b64UrlDecode(urlSpki as string);
	const pem = publicPemWrap(spki);
	const [prettyFingerprint, setPrettyFingerprint] = useState('');
	const [hexFingerprint, setHexFingerprint] = useState('');

	useEffect(() => {
		const getFingerprint = async () => {
			setHexFingerprint(await hexFingerprintApiKeySpki(spki));
			setPrettyFingerprint(await prettyFingerprintApiKeySpki(spki));
		};
		getFingerprint();
	}, []);

	// Perform all functions required to complete
	const completeRegistration = async () => {
		try {
			const keys: DeviceApiKey[] = await api.readDeviceApiKeys();
			if (keys.some(key => key.fingerprint == hexFingerprint)) {
				console.log('key already registered; sending completion signal');
				await completeDeviceKeyRegistration(hexFingerprint);
			} else {
				console.log(`failed to find an existing key with that fingerprint; adding ${hexFingerprint}`);
				await api.registerDeviceApiKey(pem);
				await completeDeviceKeyRegistration(hexFingerprint);
			}

			console.log('finished device key completion');
			alert('successfully authorized device!');
		} catch (error: any) {
			alert('failed to authorize new device!');
			console.log(`error: ${error}`);
		}
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

						<h4 className="text-m font-semibold">Fingerprint:</h4>
						<p className="mt-2 text-text-600">{`${prettyFingerprint}`}</p>

						<h4 className="text-m font-semibold">PEM:</h4>
						<p className="mt-2 text-text-600">{`${spki}`}</p>
					</div>
					<div className="mt-3 flex items-center gap-3 text-xs" >
						<button
							className="btn-secondary w-1/2 py-3 px-4"
						// onClick={closeModal}
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

export default DeviceKeyApproval;
