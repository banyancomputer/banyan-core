import React, { useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { SubmitButton } from '@components/common/SubmitButton';

import { useModal } from '@app/contexts/modals';
import { ToastNotifications } from '@app/utils/toastNotifications';
import { AdminClient } from '@/api/admin';
import { StorageHostRequest } from '@app/types';

export const CreateStorageHost = ({ client }: { client: AdminClient }) => {
	const { closeModal } = useModal();
	const navigate = useNavigate();
	const [hostName, setHostName] = useState('');
	const [url, setUrl] = useState('');
	const [availableStorage, setAvailableStorage] = useState(0);
	const isDataValid = useMemo(() => hostName.length >= 3, [hostName, hostName]);

	const changeHostName = (event: React.ChangeEvent<HTMLInputElement>) => {
		const regexp = new RegExp(/^.{0,32}$/);
		if (!regexp.test(event.target.value)) {
			return;
		}

		setHostName(event.target.value);
	};

	const changeUrl = (event: React.ChangeEvent<HTMLInputElement>) => {
		const regexp = new RegExp(/^.{0,32}$/);
		if (!regexp.test(event.target.value)) {
			return;
		}

		setUrl(event.target.value);
	};
	const changeAvailableSpace = (event: React.ChangeEvent<HTMLInputElement>) => {
		const regexp = new RegExp(/^.{0,32}$/);
		if (!regexp.test(event.target.value)) {
			return;
		}

		setAvailableStorage(Number(event.target.value));
	};

	const create = async () => {
		const storageHost: StorageHostRequest = {
			name: hostName,
			url: hostName,
			available_storage: availableStorage,
		};
		try {
			const newHost = await client.createStorageHost(storageHost);
			closeModal();
			navigate(`/`);
		} catch (error: any) {
			ToastNotifications.error(
				`Could not create storage host ${error.message}`,
				'Try again',
				create
			);
		}
	};

	return (
		<div className="w-modal flex flex-col gap-5">
			<div>
				<h4 className="text-m font-semibold ">{'New Storage Host'}</h4>
			</div>
			<div>
				<label>
					{}
					<input
						className="mt-2 input w-full h-11 py-3 px-4 rounded-md border-1 border-border-darken focus:outline-none"
						type="text"
						placeholder={'Host name'}
						value={hostName}
						onChange={changeHostName}
					/>
				</label>
			</div>
			<div>
				<label>
					{}
					<input
						className="mt-2 input w-full h-11 py-3 px-4 rounded-md border-1 border-border-darken focus:outline-none"
						type="text"
						placeholder={'Url'}
						value={url}
						onChange={changeUrl}
					/>
				</label>
			</div>
			<div>
				<label>
					{}
					<input
						className="mt-2 input w-full h-11 py-3 px-4 rounded-md border-1 border-border-darken focus:outline-none"
						type="text"
						placeholder={'Available Space'}
						value={availableStorage}
						onChange={changeAvailableSpace}
					/>
				</label>
			</div>
			<div className="flex items-center gap-3 text-xs">
				<button
					className="btn-secondary flex-grow py-3 px-4"
					onClick={closeModal}
				>
					{'Cancel'}
				</button>
				<SubmitButton text={'Create'} action={create} disabled={!isDataValid} />
			</div>
		</div>
	);
};
